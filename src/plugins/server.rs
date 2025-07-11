use crate::server::*;
use bevy::prelude::*;

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_server_client).add_systems(
            Update,
            (
                server_tick_system,
                handle_server_connection_tasks,
                handle_game_state_tasks,
                handle_game_actions,
                handle_action_response_tasks,
                handle_reconnect_requests,
            ),
        );
    }
}
