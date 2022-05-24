use bevy::prelude::*;

#[derive(Component)]
struct Concentration(f64);
#[derive(Component)]
struct Source(f64);
#[derive(Component)]
struct Neighbours(Vec<Entity>);

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_startup_system(create_grid_system)
        .add_system(source_system)
        .add_system(diffusion_system)
        .run();
}

fn create_grid_system(mut commands: Commands) {
    let nx = 4;
    let ny = 4;
    for i in 0..nx {
        for j in 0..ny {
            commands
                .spawn()
                .insert(Concentration(0.0))
                .insert(Source(0.0))
                .insert(Neighbours(neighbours))
                .id();
        }
    }
}

fn source_system() {}

fn diffusion_system() {}
