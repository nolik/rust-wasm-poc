use serde_json::json;
use state::{Entry, Filter, State};
use strum::IntoEnumIterator;
use yew::format::Json;
use yew::services::fetch::FetchTask;
use yew::services::fetch::{Request, Response};
use yew::services::storage::Area;
use yew::services::{ConsoleService, FetchService, StorageService};
use yew::web_sys::HtmlInputElement as InputElement;
use yew::{classes, html, Component, ComponentLink, Html, InputData, NodeRef, ShouldRender};
use yew::{events::KeyboardEvent, Classes};

use serde::Deserialize;

mod state;

const KEY: &str = "yew.url-clipper.self";

pub enum Msg {
    Add,
    Edit(usize),
    Update(String),
    UpdateStore(Result<Url, anyhow::Error>),
    UpdateEdit(String),
    Remove(usize),
    SetFilter(Filter),
    ToggleAll,
    ToggleEdit(usize),
    Toggle(usize),
    ClearCompleted,
    Focus,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Url {
    address: String,
}

pub struct Model {
    link: ComponentLink<Self>,
    storage: StorageService,
    state: State,
    focus_ref: NodeRef,
    fetch_task: Option<FetchTask>,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let storage = StorageService::new(Area::Local).expect("storage was disabled by the user");
        let entries = {
            if let Json(Ok(restored_model)) = storage.restore(KEY) {
                restored_model
            } else {
                Vec::new()
            }
        };
        let state = State {
            entries,
            filter: Filter::All,
            value: "".into(),
            edit_value: "".into(),
        };
        let focus_ref = NodeRef::default();
        let fetch_task = None;
        Self {
            link,
            storage,
            state,
            focus_ref,
            fetch_task,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Add => {
                let link = self.state.value.clone();
                let trimmed_link = link.trim();
                if !trimmed_link.is_empty() {
                    let task = self.fetch_data(trimmed_link);
                    // store the task so it isn't canceled immediately
                    self.fetch_task = Some(task);
                }
                self.state.value = "".to_string();
            }
            Msg::Edit(idx) => {
                ConsoleService::log("Edit");
                let edit_value = self.state.edit_value.trim().to_string();
                self.state.complete_edit(idx, edit_value);
                self.state.edit_value = "".to_string();
            }
            Msg::Update(val) => {
                ConsoleService::log("Update");
                println!("Input: {}", val);
                self.state.value = val;
            }
            Msg::UpdateEdit(val) => {
                println!("Input: {}", val);
                self.state.edit_value = val;
            }
            Msg::UpdateStore(response) => {
                ConsoleService::log("UpdateStore");

                match response {
                    Ok(url) => {
                        ConsoleService::log("ok URL");
                        let entry = Entry {
                            description: url.clone().address,
                            completed: false,
                            editing: false,
                        };
                        self.state.entries.push(entry);
                    }
                    Err(error) => {
                        ConsoleService::log("error");
                        ConsoleService::log(&error.root_cause().to_string());
                    }
                }
            }
            Msg::Remove(idx) => {
                self.state.remove(idx);
            }
            Msg::SetFilter(filter) => {
                self.state.filter = filter;
            }
            Msg::ToggleEdit(idx) => {
                self.state.edit_value = self.state.entries[idx].description.clone();
                self.state.clear_all_edit();
                self.state.toggle_edit(idx);
            }
            Msg::ToggleAll => {
                let status = !self.state.is_all_completed();
                self.state.toggle_all(status);
            }
            Msg::Toggle(idx) => {
                self.state.toggle(idx);
            }
            Msg::ClearCompleted => {
                self.state.clear_completed();
            }
            Msg::Focus => {
                if let Some(input) = self.focus_ref.cast::<InputElement>() {
                    input.focus().unwrap();
                }
            }
        }
        self.storage.store(KEY, Json(&self.state.entries));
        true
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let hidden_class = if self.state.entries.is_empty() {
            "hidden"
        } else {
            ""
        };
        html! {
            <div class="todomvc-wrapper">
                <section class="todoapp">
                    <header class="header">
                        <h1>{ "Clip the link" }</h1>
                        { self.view_input() }
                    </header>
                    <section class=classes!("main", hidden_class)>
                        <input
                            type="checkbox"
                            class="toggle-all"
                            id="toggle-all"
                            checked=self.state.is_all_completed()
                            onclick=self.link.callback(|_| Msg::ToggleAll)
                        />
                        <label for="toggle-all" />
                        <ul class="todo-list">
                            { for self.state.entries.iter().filter(|e| self.state.filter.fits(e)).enumerate().map(|e| self.view_entry(e)) }
                        </ul>
                    </section>
                    <footer class=classes!("footer", hidden_class)>
                        <span class="todo-count">
                            <strong>{ self.state.total() }</strong>
                            { " item(s) left" }
                        </span>
                        <ul class="filters">
                            { for Filter::iter().map(|flt| self.view_filter(flt)) }
                        </ul>
                        <button class="clear-completed" onclick=self.link.callback(|_| Msg::ClearCompleted)>
                            { format!("Clear completed ({})", self.state.total_completed()) }
                        </button>
                    </footer>
                </section>
                <footer class="info">
                    <p>{ "Double-click to add link" }</p>
                </footer>
            </div>
        }
    }
}

impl Model {
    fn fetch_data(&mut self, description: &str) -> FetchTask {
        let body = &json!({ "address": description });
        let post_request = Request::post("http://127.0.0.1:8090/clip")
            .header("Content-Type", "application/json")
            .body(Json(body))
            .expect("Failed to build request.");

        let callback =
            self.link
                .callback(|response: Response<Json<Result<Url, anyhow::Error>>>| {
                    let (meta, Json(data)) = response.into_parts();
                    ConsoleService::log(&meta.status.to_string());
                    Msg::UpdateStore(data)
                });

        FetchService::fetch(post_request, callback).expect("failed to start request")
    }

    fn view_filter(&self, filter: Filter) -> Html {
        let cls = if self.state.filter == filter {
            "selected"
        } else {
            "not-selected"
        };
        html! {
            <li>
                <a class=cls
                   href=filter.as_href()
                   onclick=self.link.callback(move |_| Msg::SetFilter(filter))
                >
                    { filter }
                </a>
            </li>
        }
    }

    fn view_input(&self) -> Html {
        html! {
            // You can use standard Rust comments. One line:
            // <li></li>
            <input
                class="new-todo"
                placeholder="Link to be clipped"
                value=self.state.value.clone()
                oninput=self.link.callback(|e: InputData| Msg::Update(e.value))
                onkeypress=self.link.batch_callback(|e: KeyboardEvent| {
                    if e.key() == "Enter" { Some(Msg::Add) } else { None }
                })
            />
            /* Or multiline:
            <ul>
                <li></li>
            </ul>
            */
        }
    }

    fn view_entry(&self, (idx, entry): (usize, &Entry)) -> Html {
        let mut class = Classes::from("todo");
        if entry.editing {
            class.push(" editing");
        }
        if entry.completed {
            class.push(" completed");
        }
        html! {
            <li class=class>
                <div class="view">
                    <input
                        type="checkbox"
                        class="toggle"
                        checked=entry.completed
                        onclick=self.link.callback(move |_| Msg::Toggle(idx))
                    />
                    <label ondblclick=self.link.callback(move |_| Msg::ToggleEdit(idx))>{ &entry.description }</label>
                    <button class="destroy" onclick=self.link.callback(move |_| Msg::Remove(idx)) />
                </div>
                { self.view_entry_edit_input((idx, &entry)) }
            </li>
        }
    }

    fn view_entry_edit_input(&self, (idx, entry): (usize, &Entry)) -> Html {
        if entry.editing {
            html! {
                <input
                    class="edit"
                    type="text"
                    ref=self.focus_ref.clone()
                    value=self.state.edit_value.clone()
                    onmouseover=self.link.callback(|_| Msg::Focus)
                    oninput=self.link.callback(|e: InputData| Msg::UpdateEdit(e.value))
                    onblur=self.link.callback(move |_| Msg::Edit(idx))
                    onkeypress=self.link.batch_callback(move |e: KeyboardEvent| {
                        if e.key() == "Enter" { Some(Msg::Edit(idx)) } else { None }
                    })
                />
            }
        } else {
            html! { <input type="hidden" /> }
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}
