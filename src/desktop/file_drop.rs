//! Native Wayland file offers for the winit 0.29 backend, which omits file drops.
//!
//! This receiver shares the window's display, owns its protocol objects and queue,
//! accepts copy-only URI lists, and publishes only the latest completed drop.
use std::{
    collections::HashMap,
    io::{self, Read},
    os::{
        fd::{AsFd, AsRawFd},
        unix::net::UnixStream,
    },
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use wayland_client::{
    backend::{Backend, WaylandError},
    delegate_noop, event_created_child,
    protocol::{wl_data_device, wl_data_device_manager, wl_data_offer, wl_registry, wl_seat},
    Connection, Dispatch, Proxy, QueueHandle, WEnum,
};

const MIME: &str = "text/uri-list";
type Delivery = Arc<Mutex<Option<Result<PathBuf, String>>>>;

pub(crate) struct FileDrop {
    delivery: Delivery,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl FileDrop {
    /// Starts a receiver on the existing native display.
    ///
    /// # Safety
    ///
    /// The native window/display must remain alive until this receiver is dropped.
    pub(crate) unsafe fn start(
        display: *mut std::ffi::c_void,
        ctx: egui::Context,
    ) -> io::Result<Self> {
        // SAFETY: the app stops and joins this worker before its native display closes.
        let connection =
            Connection::from_backend(unsafe { Backend::from_foreign_display(display.cast()) });
        let delivery = Delivery::default();
        let stop = Arc::new(AtomicBool::new(false));
        let (output, cancelled) = (delivery.clone(), stop.clone());
        let worker = thread::Builder::new()
            .name("repository-drop".into())
            .spawn(move || {
                if let Err(error) = receive(connection, output, &cancelled, ctx) {
                    tracing::warn!(%error, "Native file-drop receiver stopped");
                }
            })?;
        Ok(Self {
            delivery,
            stop,
            worker: Some(worker),
        })
    }

    pub(crate) fn poll(&self) -> Option<Result<PathBuf, String>> {
        self.delivery
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .take()
    }
}

impl Drop for FileDrop {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

struct Offer {
    mime: bool,
    action: wl_data_device_manager::DndAction,
}
struct Transfer {
    offer: wl_data_offer::WlDataOffer,
    pipe: UnixStream,
    bytes: Vec<u8>,
    started: Instant,
}
#[derive(Default)]
struct State {
    manager: Option<wl_data_device_manager::WlDataDeviceManager>,
    seats: Vec<wl_seat::WlSeat>,
    devices: Vec<wl_data_device::WlDataDevice>,
    offers: HashMap<wl_data_offer::WlDataOffer, Offer>,
    active: Option<wl_data_offer::WlDataOffer>,
    transfer: Option<Transfer>,
}

fn receive(
    connection: Connection,
    output: Delivery,
    stop: &AtomicBool,
    ctx: egui::Context,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut queue = connection.new_event_queue();
    connection.display().get_registry(&queue.handle(), ());
    let mut state = State::default();
    while !stop.load(Ordering::Relaxed) {
        queue.dispatch_pending(&mut state)?;
        if let Some(result) = state.read_transfer() {
            if let Some(result) = result.transpose() {
                *output.lock().unwrap_or_else(|e| e.into_inner()) = Some(result);
                ctx.request_repaint();
            }
        }
        wait_io(&connection)?;
    }
    drop(state);
    connection.flush()?;
    Ok(())
}

// libwayland synchronizes this read guard with winit's independent event queue.
// Readiness can change between poll and read, so temporary I/O is retried.
fn wait_io(connection: &Connection) -> Result<(), WaylandError> {
    let Some(guard) = connection.prepare_read() else {
        return Ok(());
    };
    let pending_write = match connection.flush() {
        Ok(()) => false,
        Err(WaylandError::Io(error))
            if matches!(
                error.kind(),
                io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
            ) =>
        {
            true
        }
        Err(error) => return Err(error),
    };
    let mut fd = libc::pollfd {
        fd: guard.connection_fd().as_raw_fd(),
        events: libc::POLLIN | if pending_write { libc::POLLOUT } else { 0 },
        revents: 0,
    };
    // SAFETY: one initialized pollfd borrowed from the live read guard.
    let ready = unsafe { libc::poll(&mut fd, 1, 100) };
    if ready < 0 && io::Error::last_os_error().kind() != io::ErrorKind::Interrupted {
        return Err(WaylandError::Io(io::Error::last_os_error()));
    }
    if ready > 0 && fd.revents & (libc::POLLIN | libc::POLLERR | libc::POLLHUP) != 0 {
        match guard.read() {
            Ok(_) => (),
            Err(WaylandError::Io(error))
                if matches!(
                    error.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
                ) => {}
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

impl State {
    fn add_devices(&mut self, qh: &QueueHandle<Self>) {
        if let Some(manager) = &self.manager {
            for seat in &self.seats[self.devices.len()..] {
                self.devices.push(manager.get_data_device(seat, qh, ()));
            }
        }
    }

    fn discard(&mut self, offer: wl_data_offer::WlDataOffer) {
        self.offers.remove(&offer);
        offer.destroy();
    }

    fn read_transfer(&mut self) -> Option<Result<Option<PathBuf>, String>> {
        let transfer = self.transfer.as_mut()?;
        let result = read_payload(&mut transfer.pipe, &mut transfer.bytes, transfer.started)?
            .map(|()| first_path(&transfer.bytes))
            .map_err(|error| format!("Could not read dropped files: {error}"));
        let transfer = self.transfer.take().unwrap();
        if result.is_ok() && transfer.offer.version() >= 3 {
            transfer.offer.finish();
        }
        self.discard(transfer.offer);
        Some(result)
    }

    fn begin_transfer(&mut self) -> io::Result<()> {
        let Some(offer) = self.active.take() else {
            return Ok(());
        };
        let accepted = self.offers.get(&offer).is_some_and(|info| {
            info.mime
                && (offer.version() < 3 || info.action == wl_data_device_manager::DndAction::Copy)
        });
        if !accepted {
            self.discard(offer);
            return Ok(());
        }
        let (pipe, write) = UnixStream::pair()?;
        pipe.set_nonblocking(true)?;
        offer.receive(MIME.into(), write.as_fd());
        if let Some(previous) = self.transfer.take() {
            self.discard(previous.offer);
        }
        self.transfer = Some(Transfer {
            offer,
            pipe,
            bytes: Vec::new(),
            started: Instant::now(),
        });
        Ok(())
    }
}

impl Drop for State {
    fn drop(&mut self) {
        for (offer, _) in self.offers.drain() {
            offer.destroy();
        }
        for device in &self.devices {
            if device.version() >= 2 {
                device.release();
            }
        }
        for seat in &self.seats {
            if seat.version() >= 5 {
                seat.release();
            }
        }
    }
}

fn read_payload(
    pipe: &mut UnixStream,
    bytes: &mut Vec<u8>,
    started: Instant,
) -> Option<io::Result<()>> {
    let mut chunk = [0; 4096];
    loop {
        if started.elapsed() >= Duration::from_secs(3) {
            return Some(Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "Dropped URI list timed out",
            )));
        }
        match pipe.read(&mut chunk) {
            Ok(0) => return Some(Ok(())),
            Ok(count) if bytes.len() + count <= 65_536 => bytes.extend_from_slice(&chunk[..count]),
            Ok(_) => {
                return Some(Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Dropped URI list exceeds 64 KiB",
                )))
            }
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => return None,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => (),
            Err(error) => return Some(Err(error)),
        }
    }
}

fn first_path(bytes: &[u8]) -> Option<PathBuf> {
    let text = std::str::from_utf8(bytes).ok()?;
    let first = text
        .lines()
        .find(|line| !line.trim().is_empty() && !line.starts_with('#'))?;
    url::Url::parse(first).ok()?.to_file_path().ok()
}

impl Dispatch<wl_registry::WlRegistry, ()> for State {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            match interface.as_str() {
                "wl_data_device_manager" => {
                    state.manager = Some(registry.bind(name, version.min(3), qh, ()))
                }
                "wl_seat" => state
                    .seats
                    .push(registry.bind(name, version.min(5), qh, ())),
                _ => return,
            }
            state.add_devices(qh);
        }
    }
}
impl Dispatch<wl_data_device::WlDataDevice, ()> for State {
    fn event(
        state: &mut Self,
        _: &wl_data_device::WlDataDevice,
        event: wl_data_device::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            wl_data_device::Event::DataOffer { id } => {
                state.offers.insert(
                    id,
                    Offer {
                        mime: false,
                        action: wl_data_device_manager::DndAction::empty(),
                    },
                );
            }
            wl_data_device::Event::Enter { serial, id, .. } => {
                if let Some(old) = state.active.take() {
                    state.discard(old);
                }
                if let Some(offer) = id {
                    let mime = state.offers.get(&offer).is_some_and(|info| info.mime);
                    offer.accept(serial, mime.then(|| MIME.to_owned()));
                    if mime && offer.version() >= 3 {
                        offer.set_actions(
                            wl_data_device_manager::DndAction::Copy,
                            wl_data_device_manager::DndAction::Copy,
                        );
                    }
                    state.active = Some(offer);
                }
            }
            wl_data_device::Event::Leave => {
                if let Some(offer) = state.active.take() {
                    state.discard(offer);
                }
            }
            wl_data_device::Event::Drop => {
                if let Err(error) = state.begin_transfer() {
                    tracing::warn!(%error, "Could not receive dropped files");
                }
            }
            wl_data_device::Event::Selection { id: Some(offer) } => state.discard(offer),
            _ => (),
        }
    }
    event_created_child!(State, wl_data_device::WlDataDevice, [0 => (wl_data_offer::WlDataOffer, ())]);
}
impl Dispatch<wl_data_offer::WlDataOffer, ()> for State {
    fn event(
        state: &mut Self,
        offer: &wl_data_offer::WlDataOffer,
        event: wl_data_offer::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let Some(info) = state.offers.get_mut(offer) {
            match event {
                wl_data_offer::Event::Offer { mime_type } => info.mime |= mime_type == MIME,
                wl_data_offer::Event::Action {
                    dnd_action: WEnum::Value(action),
                } => info.action = action,
                _ => (),
            }
        }
    }
}
delegate_noop!(State: ignore wl_seat::WlSeat);
delegate_noop!(State: ignore wl_data_device_manager::WlDataDeviceManager);

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn uri_lists_preserve_the_first_local_url_and_original_path_bytes() {
        let path = PathBuf::from("/tmp/drop 漢字 ;/name #percent%.rs");
        let url = url::Url::from_file_path(&path).unwrap();
        let list = format!("# desktop URI list\r\n\r\n{url}\r\nfile:///another/folder\r\n");
        assert_eq!(first_path(list.as_bytes()), Some(path));
        assert_eq!(
            first_path(b"file://localhost/tmp/repo\n"),
            Some(PathBuf::from("/tmp/repo"))
        );
        for invalid in [
            &b"https://example.com/repo\nfile:///tmp/fallback"[..],
            &b"file://remote-host/tmp/repo"[..],
            &b"/tmp/plain-path"[..],
            &b"# comment only\r\n"[..],
            &b"\xfffile:///tmp/repo"[..],
        ] {
            assert!(first_path(invalid).is_none(), "{invalid:?}");
        }
    }

    #[test]
    fn offer_reads_are_incremental_and_bounded_by_size_and_deadline() {
        let (mut pipe, mut writer) = UnixStream::pair().unwrap();
        pipe.set_nonblocking(true).unwrap();
        let mut bytes = Vec::new();
        let start = Instant::now();
        writer.write_all(b"file:///tmp/space%20").unwrap();
        assert!(read_payload(&mut pipe, &mut bytes, start).is_none());
        assert_eq!(bytes, b"file:///tmp/space%20");
        writer.write_all(b"folder\r\n").unwrap();
        drop(writer);
        read_payload(&mut pipe, &mut bytes, start).unwrap().unwrap();
        assert_eq!(first_path(&bytes), Some(PathBuf::from("/tmp/space folder")));
        let (mut pipe, mut writer) = UnixStream::pair().unwrap();
        pipe.set_nonblocking(true).unwrap();
        let mut bytes = Vec::new();
        assert_eq!(
            read_payload(&mut pipe, &mut bytes, start - Duration::from_secs(4))
                .unwrap()
                .unwrap_err()
                .kind(),
            io::ErrorKind::TimedOut
        );
        writer.write_all(&vec![b'x'; 65_537]).unwrap();
        assert_eq!(
            read_payload(&mut pipe, &mut bytes, Instant::now())
                .unwrap()
                .unwrap_err()
                .kind(),
            io::ErrorKind::InvalidData
        );
        assert!(bytes.len() <= 65_536);
    }
}
