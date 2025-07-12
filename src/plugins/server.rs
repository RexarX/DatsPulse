use crate::server::*;
use crate::types::*;
use bevy::prelude::*;

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add server-specific events
            .add_event::<ApiArenaEvent>()
            .add_event::<ApiMoveEvent>()
            .add_event::<ApiRegistrationEvent>()
            .add_event::<ConnectionEvent>()
            .add_event::<ReconnectRequestEvent>()
            // Add server systems
            .add_systems(Startup, (setup_server_client, setup_rate_limiter))
            .add_systems(
                Update,
                (
                    server_tick_system,
                    handle_registration_tasks,
                    handle_arena_state_tasks,
                    handle_game_move_commands,
                    handle_move_commands,
                    handle_move_response_tasks,
                    handle_logs_requests,
                    handle_logs_response_tasks,
                    handle_reconnect_requests,
                    monitor_connection_system,
                    auto_move_system,
                ),
            );
    }
}
