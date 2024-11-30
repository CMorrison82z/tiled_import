use bevy::scene::SceneBundle;

pub(crate) fn clone_scene(sb: &SceneBundle) -> SceneBundle {
    SceneBundle {
        scene: sb.scene.clone(),
        transform: sb.transform.clone(),
        global_transform: sb.global_transform.clone(),
        visibility: sb.visibility.clone(),
        inherited_visibility: sb.inherited_visibility.clone(),
        view_visibility: sb.view_visibility.clone(),
    }
}
