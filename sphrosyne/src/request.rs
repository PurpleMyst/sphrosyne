use vigem_client_c::X360State;

pub(crate) enum PadRequest {
    NewID,
    Discard(usize),
    Update(usize, X360State),
}
