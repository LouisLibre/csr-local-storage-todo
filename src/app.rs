use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Title text="CSR Local Storage Todo App"/>
        <Style>r#"
        button { margin-left: 8px;}
        .done { color: gray; text-decoration: line-through;}
        "#
        </Style>
        <Router>
        <Routes>
        <Route path="" view=HomePage/>
        </Routes>
        </Router>
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Bucket {
    Todo,
    Done,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: u32,
    pub text: String,
    pub bucket: Bucket,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub uncommitted_todo: String,
    pub todos: Vec<Todo>,
}

const STORAGE_KEY: &str = "app-state";

impl Default for AppState {
    fn default() -> Self {
        let starting_todos = window().local_storage().ok().flatten().and_then(|storage| {
            storage
                .get_item(STORAGE_KEY)
                .ok()
                .flatten()
                .and_then(|value| serde_json::from_str::<AppState>(&value).ok())
        });

        match starting_todos {
            Some(todos) => {
                logging::log!("starting_todos: {:?}", todos);
                todos
            }
            None => AppState {
                uncommitted_todo: String::new(),
                todos: Vec::<Todo>::new(),
            },
        }
    }
}

#[component]
fn HomePage() -> impl IntoView {
    let app_state = create_rw_signal(AppState::default());

    let (uncommitted_todo, set_uncommitted_todo) = create_slice(
        app_state,
        |state| state.uncommitted_todo.clone(),
        |state, new_value: String| {
            logging::log!("set_uncommitted_todo: {:?}", new_value);
            state.uncommitted_todo = new_value.clone();
        },
    );

    let (todos, set_todos) = create_slice(
        app_state,
        |state| state.todos.clone(),
        |state, new_value: Vec<Todo>| {
            logging::log!("set_todos: {:?}", new_value);
            state.todos = new_value.clone();
        },
    );

    let save_to_local_storage = move || {
        if let Ok(Some(storage)) = window().local_storage() {
            let state = AppState {
                uncommitted_todo: app_state.get().uncommitted_todo.clone(),
                todos: app_state.get().todos.clone(),
            };
            let json = serde_json::to_string(&state).expect("couldn't serialize Todos");
            if storage.set_item(STORAGE_KEY, &json).is_err() {
                logging::error!(
                    "save_to_local_storage: error while trying to set item in localStorage"
                );
            }
        }
    };

    let _use_effect_local_storage = watch(
        move || app_state.get(),
        move |new_todo_list, old_todo_list, _| {
            logging::log!("new_todo_list: {:?}", new_todo_list);
            logging::log!("old_todo_list: {:?}", old_todo_list);
            save_to_local_storage();
        },
        false,
    );

    let todo_list = move || {
        todos
            .get()
            .iter()
            .filter(|todo| todo.bucket == Bucket::Todo)
            .cloned()
            .collect::<Vec<Todo>>()
    };

    let done_list = move || {
        todos
            .get()
            .iter()
            .filter(|todo| todo.bucket == Bucket::Done)
            .cloned()
            .collect::<Vec<Todo>>()
    };

    let input_element: NodeRef<html::Input> = create_node_ref();

    let on_click = move |_| {
        logging::log!("clicked A");
        let value = input_element().expect("<input> should be mounted").value();
        logging::log!("value: {:?}", value);

        app_state.update(move |state| {
            state.todos.push(Todo {
                id: state.todos.len() as u32, // Simple ID generation
                text: value.to_string(),
                bucket: Bucket::Todo, // New todos start in the Soon bucket
            });
        });
    };

    let mark_as_todo = move |index: u32| {
        logging::log!("move to done");
        // TODO: Implement logic to move a todo from the curent bucket to the Soon bucket
        app_state.update(|state| {
            if let Some(todo) = state.todos.iter_mut().find(|todo| todo.id == index) {
                todo.bucket = Bucket::Todo;
            }
        });
    };

    let mark_as_done = move |index: u32| {
        logging::log!("move to done");
        // TODO: Implement logic to move a todo from the current bucket to the Done bucket
        app_state.update(|state| {
            if let Some(todo) = state.todos.iter_mut().find(|todo| todo.id == index) {
                todo.bucket = Bucket::Done;
            }
        });
    };

    view! {
      <h1>CSR Local Storage Todo App</h1>
      <input
          type="text"
          placeholder="Add todo"
          prop:value=uncommitted_todo
          on:input=move |ev| {
              let new_value = event_target_value(&ev);
              //logging::log!("wtf: {:?}", new_value);
              set_uncommitted_todo.set(new_value);
          }
          node_ref=input_element
      />
      <button
      on:click=on_click
      >Add todo</button>
      <h2>Todo List</h2>
      <div>
      {move || todo_list().into_iter().map(|todo| view! {
          <input
              type="checkbox"
              name="todo"
              on:input=move |_| mark_as_done(todo.id)
          /> {todo.text}
          <br/>
      }).collect_view()}
      </div>
      <h2>Done List</h2>
      <div>
          {move || done_list().into_iter().map(|todo| view! {
              <span class="done"> {todo.text} </span>
              <button on:click=move |_| mark_as_todo(todo.id)>UNDO</button>
              <br/>
          }).collect_view()}
      </div>
    }
}
