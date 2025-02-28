use re_data_store::InstancePath;
use re_log_types::ComponentPath;
use re_query::{get_component_with_instances, QueryError};

use crate::{misc::ViewerContext, ui::UiVerbosity};

use super::DataUi;

const HIDDEN_COMPONENTS_FOR_ALL_VERBOSITY: &[&str] = &["rerun.instance_key"];
const HIDDEN_COMPONENTS_FOR_LOW_VERBOSITY: &[&str] = &[];

impl DataUi for InstancePath {
    fn data_ui(
        &self,
        ctx: &mut ViewerContext<'_>,
        ui: &mut egui::Ui,
        verbosity: UiVerbosity,
        query: &re_arrow_store::LatestAtQuery,
    ) {
        let store = &ctx.log_db.entity_db.data_store;

        let Some(mut components) = store.all_components(&query.timeline, &self.entity_path) else {
            ui.label(format!("No components in entity {}", self.entity_path));
            return;
        };
        components.sort();

        egui::Grid::new("entity_instance")
            .num_columns(2)
            .show(ui, |ui| {
                for component_name in components {
                    let component_data = get_component_with_instances(
                        store,
                        query,
                        &self.entity_path,
                        component_name,
                    );

                    if matches!(component_data, Err(QueryError::PrimaryNotFound)) {
                        continue; // no need to show components that are unset at this point in time
                    }

                    // Certain fields are hidden.
                    if HIDDEN_COMPONENTS_FOR_ALL_VERBOSITY.contains(&component_name.as_str()) {
                        continue;
                    }
                    match verbosity {
                        UiVerbosity::Small | UiVerbosity::Reduced => {
                            if HIDDEN_COMPONENTS_FOR_LOW_VERBOSITY
                                .contains(&component_name.as_str())
                            {
                                continue;
                            }
                        }
                        UiVerbosity::All => {}
                    }

                    ctx.component_path_button_to(
                        ui,
                        component_name.short_name(),
                        &ComponentPath::new(self.entity_path.clone(), component_name),
                    );

                    match component_data {
                        Err(err) => {
                            ui.label(ctx.re_ui.error_text(format!("Error: {err}")));
                        }
                        Ok((_, component_data)) => {
                            if self.instance_key.is_splat() {
                                super::component::EntityComponentWithInstances {
                                    entity_path: self.entity_path.clone(),
                                    component_data,
                                }
                                .data_ui(
                                    ctx,
                                    ui,
                                    UiVerbosity::Small,
                                    query,
                                );
                            } else {
                                ctx.component_ui_registry.ui(
                                    ctx,
                                    ui,
                                    UiVerbosity::Small,
                                    query,
                                    &component_data,
                                    &self.instance_key,
                                );
                            }
                        }
                    }

                    ui.end_row();
                }
                Some(())
            });
    }
}
