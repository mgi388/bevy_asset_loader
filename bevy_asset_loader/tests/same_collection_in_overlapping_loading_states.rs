use bevy::asset::UntypedHandle;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy_asset_loader::prelude::*;

/// Two loading states load the same collection, and the first one is still loading when the second
/// one starts. Whichever loading state checks the shared collection first, the second one still
/// finishes.
#[test]
fn same_collection_in_overlapping_loading_states() {
    for first_checks_first in [true, false] {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default(), StatesPlugin));
        app.init_asset::<NeverLoadedAsset>();
        app.init_state::<FirstState>();
        app.init_state::<SecondState>();
        app.add_loading_state(
            LoadingState::new(FirstState::Loading)
                .continue_to_state(FirstState::Done)
                .load_collection::<SharedAssets>()
                .load_collection::<NeverLoadedAssets>(),
        )
        .add_loading_state(
            LoadingState::new(SecondState::Loading)
                .continue_to_state(SecondState::Done)
                .load_collection::<SharedAssets>(),
        );
        let first = LoadingStateSet(FirstState::Loading);
        let second = LoadingStateSet(SecondState::Loading);
        if first_checks_first {
            app.configure_sets(Update, first.before(second));
        } else {
            app.configure_sets(Update, second.before(first));
        }

        app.world_mut()
            .resource_mut::<NextState<FirstState>>()
            .set(FirstState::Loading);
        for _ in 0..10 {
            app.update();
        }
        app.world_mut()
            .resource_mut::<NextState<SecondState>>()
            .set(SecondState::Loading);
        for _ in 0..10 {
            app.update();
        }

        assert_eq!(
            app.world().resource::<State<SecondState>>().get(),
            &SecondState::Done,
            "first_checks_first: {first_checks_first}"
        );
        assert_eq!(
            app.world().resource::<State<FirstState>>().get(),
            &FirstState::Loading,
            "first_checks_first: {first_checks_first}"
        );
    }
}

/// A collection without assets, so it is loaded at its first check.
#[derive(Resource)]
struct SharedAssets;

impl AssetCollection for SharedAssets {
    fn create(_world: &mut World) -> Self {
        Self
    }

    fn load(_world: &mut World) -> Vec<UntypedHandle> {
        Vec::new()
    }
}

/// A collection with an asset that never loads, so its loading state keeps loading.
#[derive(Resource)]
struct NeverLoadedAssets;

impl AssetCollection for NeverLoadedAssets {
    fn create(_world: &mut World) -> Self {
        Self
    }

    fn load(world: &mut World) -> Vec<UntypedHandle> {
        vec![
            world
                .resource::<Assets<NeverLoadedAsset>>()
                .reserve_handle()
                .untyped(),
        ]
    }
}

#[derive(Asset, TypePath)]
struct NeverLoadedAsset;

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum FirstState {
    #[default]
    Idle,
    Loading,
    Done,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum SecondState {
    #[default]
    Idle,
    Loading,
    Done,
}
