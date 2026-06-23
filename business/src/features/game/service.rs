use crate::prelude::*;
use super::*;

#[derive(Clone)]
pub struct Service {
    internal_repo: Arc<dyn InternalRepository>,
    external_repo: Arc<dyn ExternalRepository>,
}

impl Service {
    pub fn new(
        internal_repo: Arc<dyn InternalRepository>,
        external_repo: Arc<dyn ExternalRepository>,
    ) -> Self {
        Self { internal_repo, external_repo }
    }
}

#[async_trait]
impl UseCase for Service {
    async fn get_internal_game(&self, id: &Uuid) -> Result<Option<Game>, UseCaseError> {
        let full_game = self.internal_repo
            .get_game(id)
            .await?;

        Ok(full_game)
    }

    async fn get_external_game(&self, id: u64) -> Result<Option<Game>, UseCaseError> {
        // Get the game from the external repository
        let mut game = match self.external_repo
            .get_game(id)
            .await?
        {
            Some(game) => game,
            None => return Ok(None),
        };

        // Get the game's ID from the internal repository
        let internal_game_id = self.internal_repo
            .sync_db_game_from_external(&mut game)
            .await?;

        // Resolve the game's ID into the full game data
        let full_game = self.internal_repo
            .get_game(&internal_game_id)
            .await?;

        Ok(full_game)
    }
}