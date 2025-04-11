use bevy::prelude::*;
use bevy_hanabi::prelude::*;

use crate::assets::FxAssets;

pub(crate) fn register_fx(
    mut effects: ResMut<Assets<EffectAsset>>,
    mut fx_assets: ResMut<FxAssets>,
) {
    let effect_handle = effects.add(laser_hit_effect());
    fx_assets.laser_hit_vfx = effect_handle;

    let effect_handle = effects.add(ship_destroy_effect());
    fx_assets.ship_destroy_vfx = effect_handle;
}

fn ship_destroy_effect() -> EffectAsset {
    // Define a color gradient from red to transparent black
    let mut gradient = Gradient::new();
    gradient.add_key(0.0, Vec4::new(1., 0., 0., 1.));
    gradient.add_key(1.0, Vec4::splat(0.0));

    let mut module = Module::default();
    
    // On spawn, randomly initialize the position of the particle
    // to be over the surface of a sphere of radius 2 units.
    let init_pos = SetPositionSphereModifier {
        center: module.lit(Vec3::ZERO),
        radius: module.lit(4.),
        dimension: ShapeDimension::Volume,
    };

    let vel_mod = SetVelocitySphereModifier {
        center: module.lit(Vec3::ZERO),
        speed: module.lit(6.),
    };

    // Initialize the total lifetime of the particle, that is
    // the time for which it's simulated and rendered. This modifier
    // is almost always required, otherwise the particles won't show.
    let lifetime = module.lit(1.0); // literal value "10.0"
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    // Every frame, add a gravity-like acceleration downward
    let accel = module.lit(Vec3::new(0., -10., 0.));
    let update_accel = AccelModifier::new(accel);

    let scale = module.lit(Vec3::splat(0.2));
    let init_scale = SetAttributeModifier::new(Attribute::SIZE3, scale);


    // Create the effect asset
    EffectAsset::new(
        // Maximum number of particles alive at a time
        32768,
        SpawnerSettings::once(300.0.into()),
        // Move the expression module into the asset
        module,
    )
    .with_name("MyEffect")
    .init(init_pos)
    .init(init_lifetime)
    .init(init_scale)
    // .init(init_velocity)
    .init(vel_mod)
    .update(update_accel)
    // Render the particles with a color gradient over their
    // lifetime. This maps the gradient key 0 to the particle spawn
    // time, and the gradient key 1 to the particle death (10s).
    .render(ColorOverLifetimeModifier {
        gradient,
        ..default()
    })
    .render(OrientModifier {
        mode: OrientMode::ParallelCameraDepthPlane,
        //rotation: Some(rotation_attr),
        ..default()
    })
}

fn laser_hit_effect() -> EffectAsset {
        // Define a color gradient from green to transparent black
        let mut gradient = Gradient::new();
        gradient.add_key(0.0, Vec4::new(0., 1., 0., 1.));
        gradient.add_key(1.0, Vec4::splat(0.0));
    
        let mut module = Module::default();
        
        // On spawn, randomly initialize the position of the particle
        // to be over the surface of a sphere of radius 2 units.
        let init_pos = SetPositionSphereModifier {
            center: module.lit(Vec3::ZERO),
            radius: module.lit(1.),
            dimension: ShapeDimension::Volume,
        };
    
        let vel_mod = SetVelocitySphereModifier {
            center: module.lit(Vec3::ZERO),
            speed: module.lit(6.),
        };
    
        // Initialize the total lifetime of the particle, that is
        // the time for which it's simulated and rendered. This modifier
        // is almost always required, otherwise the particles won't show.
        let lifetime = module.lit(1.0); // literal value "10.0"
        let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);
    
        // Every frame, add a gravity-like acceleration downward
        let accel = module.lit(Vec3::new(0., -10., 0.));
        let update_accel = AccelModifier::new(accel);
    
        let scale = module.lit(Vec3::splat(0.2));
        let init_scale = SetAttributeModifier::new(Attribute::SIZE3, scale);
    
    
        // Create the effect asset
        EffectAsset::new(
            // Maximum number of particles alive at a time
            32768,
            SpawnerSettings::once(50.0.into()),
            // Move the expression module into the asset
            module,
        )
        .with_name("MyEffect")
        .init(init_pos)
        .init(init_lifetime)
        .init(init_scale)
        // .init(init_velocity)
        .init(vel_mod)
        .update(update_accel)
        // Render the particles with a color gradient over their
        // lifetime. This maps the gradient key 0 to the particle spawn
        // time, and the gradient key 1 to the particle death (10s).
        .render(ColorOverLifetimeModifier {
            gradient,
            ..default()
        })
        .render(OrientModifier {
            mode: OrientMode::ParallelCameraDepthPlane,
            //rotation: Some(rotation_attr),
            ..default()
        })
}


// pub(crate) fn register_fx(
//     mut effects: ResMut<Assets<EffectAsset>>,
//     mut fx_assets: ResMut<FxAssets>,
// ) {
//     // Define a color gradient from red to transparent black
//     let mut gradient = Gradient::new();
//     gradient.add_key(0.0, Vec4::new(0., 1., 0., 1.));
//     gradient.add_key(1.0, Vec4::splat(0.0));

//     let writer = ExprWriter::new();
    
//     // On spawn, randomly initialize the position of the particle
//     // to be over the surface of a sphere of radius 2 units.
//     let init_pos = SetPositionSphereModifier {
//         center: writer.lit(Vec3::ZERO).expr(),
//         radius: writer.lit(1.).expr(),
//         dimension: ShapeDimension::Volume,
//     };

//     let vel_mod = SetVelocitySphereModifier {
//         center: writer.lit(Vec3::ZERO).expr(),
//         speed: writer.lit(6.).expr(),
//     };

//     // Initialize the total lifetime of the particle, that is
//     // the time for which it's simulated and rendered. This modifier
//     // is almost always required, otherwise the particles won't show.
//     let lifetime = writer.lit(1.0).expr(); // literal value "10.0"
//     let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

//     let hit_direction = writer.add_property("hit_direction", Value::Vector(VectorValue::new_vec3(-Vec3::Z)));
//     let hit_direction_expr = writer.prop(hit_direction).expr();

//     // let velocity = writer.prop(hit_direction).mul(writer.lit(50.)).expr();
//     // let init_velocity = SetAttributeModifier::new(Attribute::VELOCITY, velocity);

//     // Every frame, add a gravity-like acceleration downward
//     let accel = writer.lit(Vec3::new(0., -10., 0.)).expr();
//     let update_accel = AccelModifier::new(accel);

//     let scale = writer.lit(Vec3::splat(0.2)).expr();
//     let init_scale = SetAttributeModifier::new(Attribute::SIZE3, scale);

//     //let rotation_attr = writer.attr(Attribute::F32_0).expr();

//     // Create the effect asset
//     let effect = EffectAsset::new(
//         // Maximum number of particles alive at a time
//         32768,
//         SpawnerSettings::once(100.0.into()),
//         // Move the expression module into the asset
//         writer.finish(),
//     )
//     .with_name("MyEffect")
//     .init(init_pos)
//     .init(init_lifetime)
//     .init(init_scale)
//     // .init(init_velocity)
//     .init(vel_mod)
//     .update(update_accel)
//     // Render the particles with a color gradient over their
//     // lifetime. This maps the gradient key 0 to the particle spawn
//     // time, and the gradient key 1 to the particle death (10s).
//     .render(ColorOverLifetimeModifier {
//         gradient,
//         ..default()
//     })
//     .render(OrientModifier {
//         mode: OrientMode::FaceCameraPosition,
//         //rotation: Some(rotation_attr),
//         ..default()
//     });

//     // Insert into the asset system
//     let effect_handle = effects.add(effect);
//     fx_assets.laser_hit_vfx = effect_handle;
// }
