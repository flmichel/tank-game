use sqlx::PgPool;

pub trait RepoKind {}

#[derive(Clone)]
pub struct Game;
impl RepoKind for Game {}

#[derive(Clone)]
pub struct Repo<RepoKind> {
    pub poll: PgPool,
    kind: std::marker::PhantomData<RepoKind>,
}

impl<K: RepoKind> Repo<K> {
    pub fn new(poll: PgPool) -> Repo<K> {
        Repo {
            poll,
            kind: std::marker::PhantomData::<K>,
        }
    }
}
