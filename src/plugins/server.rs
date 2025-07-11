use crate::server::*;
use bevy::prelude::*;

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_server_client).add_systems(
            Update,
            (
                server_tick_system,
                handle_registration_tasks,
                handle_arena_state_tasks,
                handle_move_response_tasks,
                handle_move_commands,
                handle_register_requests,
                handle_reconnect_requests,
            ),
        );
    }
}
