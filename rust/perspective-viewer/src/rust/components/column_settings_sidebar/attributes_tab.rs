// ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
// ┃ ██████ ██████ ██████       █      █      █      █      █ █▄  ▀███ █       ┃
// ┃ ▄▄▄▄▄█ █▄▄▄▄▄ ▄▄▄▄▄█  ▀▀▀▀▀█▀▀▀▀▀ █ ▀▀▀▀▀█ ████████▌▐███ ███▄  ▀█ █ ▀▀▀▀▀ ┃
// ┃ █▀▀▀▀▀ █▀▀▀▀▀ █▀██▀▀ ▄▄▄▄▄ █ ▄▄▄▄▄█ ▄▄▄▄▄█ ████████▌▐███ █████▄   █ ▄▄▄▄▄ ┃
// ┃ █      ██████ █  ▀█▄       █ ██████      █      ███▌▐███ ███████▄ █       ┃
// ┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫
// ┃ Copyright (c) 2017, the Perspective Authors.                              ┃
// ┃ ╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌ ┃
// ┃ This file is part of the Perspective library, distributed under the terms ┃
// ┃ of the [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0). ┃
// ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛

use wasm_bindgen::JsValue;
use yew::{function_component, html, Callback, Html, Properties};

use crate::components::expression_editor::ExpressionEditor;
use crate::components::viewer::ColumnLocator;
use crate::config::ViewConfigUpdate;
use crate::model::UpdateAndRender;
use crate::renderer::Renderer;
use crate::session::Session;
use crate::utils::ApiFuture;
use crate::{derive_model, html_template};

#[derive(PartialEq, Clone, Properties)]
pub struct AttributesTabProps {
    pub selected_column: ColumnLocator,
    pub on_close: Callback<()>,
    pub session: Session,
    pub renderer: Renderer,
}
derive_model!(Renderer, Session for AttributesTabProps);

#[function_component]
pub fn AttributesTab(p: &AttributesTabProps) -> Html {
    let is_validating = yew::use_state_eq(|| false);
    let on_save = yew::use_callback(
        |v, p| {
            match &p.selected_column {
                ColumnLocator::Expr(None) => save_expr(v, p),
                ColumnLocator::Expr(Some(alias)) => update_expr(alias, &v, p),
                _ => {
                    tracing::error!("Tried to save expression in bad state!")
                }
            }

            p.on_close.emit(());
        },
        p.clone(),
    );

    let on_validate = yew::use_callback(
        |b, validating| {
            validating.set(b);
        },
        is_validating.setter(),
    );

    let on_delete = yew::use_callback(
        |(), p| {
            if let ColumnLocator::Expr(Some(ref s)) = p.selected_column {
                delete_expr(s, p);
            }

            p.on_close.emit(());
        },
        p.clone(),
    );

    let alias = yew::use_memo(
        |s| match s {
            ColumnLocator::Expr(Some(s)) => Some(s.clone()),
            _ => None,
        },
        p.selected_column.clone(),
    );
    html_template! {
        if matches!(p.selected_column, ColumnLocator::Expr(_)) {
            <div class="item_title">{"Expression Editor"}</div>
            <ExpressionEditor
                { on_save }
                { on_validate }
                { on_delete }
                session = { &p.session }
                alias = { (*alias).clone() }
            />
        }
    }
}
fn update_expr(name: &str, new_expression: &JsValue, props: &AttributesTabProps) {
    let n = name.to_string();
    let exp = new_expression.clone();
    let sesh = props.session.clone();
    let props = props.clone();
    ApiFuture::spawn(async move {
        let update = sesh.create_replace_expression_update(&n, &exp).await;
        props.update_and_render(update).await?;
        Ok(())
    });
}

fn save_expr(expression: JsValue, props: &AttributesTabProps) {
    let task = {
        let expression = expression.as_string().unwrap();
        let mut expressions = props.session.get_view_config().expressions.clone();
        expressions.retain(|x| x != &expression);
        expressions.push(expression);
        props.update_and_render(ViewConfigUpdate {
            expressions: Some(expressions),
            ..Default::default()
        })
    };

    ApiFuture::spawn(task);
}

fn delete_expr(expr_name: &str, props: &AttributesTabProps) {
    let session = &props.session;
    let expression = session
        .metadata()
        .get_expression_by_alias(expr_name)
        .unwrap();

    let mut expressions = session.get_view_config().expressions.clone();
    expressions.retain(|x| x != &expression);
    let config = ViewConfigUpdate {
        expressions: Some(expressions),
        ..ViewConfigUpdate::default()
    };

    let task = props.update_and_render(config);
    ApiFuture::spawn(task);
}
