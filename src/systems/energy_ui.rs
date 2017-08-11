use specs::{Entity, Join, ReadStorage, WriteStorage, System};
use components::{EnergyBar, Player, Transform};

pub struct EnergyUi {
    pub player_entity: Entity,
}

impl<'a> System<'a> for EnergyUi {
    type SystemData = (
        ReadStorage<'a, EnergyBar>,
        ReadStorage<'a, Player>,
        WriteStorage<'a, Transform>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (energy_bar_storage, player_storage, mut transform_storage) = data;

        let player = match player_storage.get(self.player_entity) {
            Some(player) => player,
            None => return,
        };

        for (_, transform) in (&energy_bar_storage, &mut transform_storage).join() {
            transform.size.x = (EnergyBar::get_max_width() as f32 * player.get_energy_percentage()) as u16;
        }
    }
}