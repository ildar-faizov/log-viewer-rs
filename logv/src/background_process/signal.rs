pub enum Signal<M, R> {
    Custom(M),
    Progress(u8),
    Complete(R),
}
