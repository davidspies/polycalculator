use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::Zero;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{
    Element, Event, HtmlElement, HtmlInputElement, HtmlSelectElement, HtmlTextAreaElement,
    KeyboardEvent,
};

mod basis;
mod format;
mod parse;
mod pascal;
mod polynomial;

use crate::basis::Basis;
use crate::parse::parse;
use crate::pascal::generate_pascal_triangle;
use crate::polynomial::Polynomial;

struct HistoryEntry {
    query: String,
    result: String,
}

struct AppState {
    // Calculator State
    history: Vec<HistoryEntry>,
    current_poly: Option<Polynomial>,
    basis: Basis,
    // Pascal's Triangle State
    pascal_rows: usize,
    pascal_triangle: Vec<Vec<BigInt>>,
    highlighted_cells: HashSet<(usize, usize)>,
}

fn update_history_display(history: &[HistoryEntry], history_list_element: &Element, basis: Basis) {
    let document = web_sys::window().unwrap().document().unwrap();
    history_list_element.set_inner_html("");

    for entry in history.iter().rev() {
        let entry_div = document.create_element("div").unwrap();
        entry_div.set_class_name("history-entry");

        let query_div = document.create_element("div").unwrap();
        query_div.set_class_name("history-query");
        query_div.set_text_content(Some(&entry.query));

        let result_div = document.create_element("div").unwrap();
        result_div.set_class_name("history-result");
        // Re-parse the original query to format it in the current basis
        if let Ok(poly) = parse(&entry.query) {
            result_div.set_text_content(Some(&basis.format(&poly)));
        } else {
            result_div.set_text_content(Some(&entry.result)); // Show original result if it was an error
        }

        entry_div.append_child(&query_div).unwrap();
        entry_div.append_child(&result_div).unwrap();
        history_list_element.append_child(&entry_div).unwrap();
    }
}

fn rerender_result(app_state: &AppState, result_output: &Element, history_list_element: &Element) {
    if let Some(poly) = &app_state.current_poly {
        result_output.set_text_content(Some(&app_state.basis.format(poly)));
    }
    update_history_display(&app_state.history, history_list_element, app_state.basis);
}

fn perform_calculation(
    input_element: &HtmlTextAreaElement,
    result_output: &Element,
    app_state: &mut AppState,
    history_list_element: &Element,
) {
    let expression_str = input_element.value();
    if expression_str.is_empty() {
        result_output.set_text_content(Some("0"));
        app_state.current_poly = Some(Polynomial::constant(BigRational::zero()));
        rerender_result(app_state, result_output, history_list_element);
        return;
    }

    let (result_text, new_poly) = match parse(&expression_str) {
        Ok(poly) => (app_state.basis.format(&poly), Some(poly)),
        Err(e) => (format!("Error: {}", e), None),
    };

    app_state.current_poly = new_poly;

    // We store the query, and the result as a string, but we'll re-format it live when basis changes
    app_state.history.push(HistoryEntry {
        query: expression_str,
        result: result_text.clone(), // This is just for non-poly results like errors
    });
    if app_state.history.len() > 10 {
        app_state.history.remove(0);
    }

    rerender_result(app_state, result_output, history_list_element);
}

fn perform_evaluation(eval_input: &HtmlInputElement, eval_result: &Element, app_state: &AppState) {
    let x_str = eval_input.value();
    if x_str.is_empty() {
        eval_result.set_text_content(Some(""));
        return;
    }

    let x_val = match x_str.parse::<BigRational>() {
        Ok(val) => val,
        Err(_) => {
            eval_result.set_text_content(Some("Invalid number for x"));
            return;
        }
    };

    if let Some(poly) = &app_state.current_poly {
        let result = poly.eval(&x_val);
        eval_result.set_text_content(Some(&result.to_string()));
    } else {
        eval_result.set_text_content(Some("No valid polynomial to evaluate."));
    }
}

fn render_pascal_triangle(state: &AppState, container: &Element) {
    let document = web_sys::window().unwrap().document().unwrap();
    container.set_inner_html("");
    for (r, row) in state.pascal_triangle.iter().enumerate() {
        let row_div = document.create_element("div").unwrap();
        row_div.set_class_name("pascal-row");
        for (c, cell_val) in row.iter().enumerate() {
            let cell_div = document.create_element("div").unwrap();
            cell_div.set_class_name("pascal-cell");
            if state.highlighted_cells.contains(&(r, c)) {
                cell_div.class_list().add_1("highlighted").unwrap();
            }
            cell_div.set_attribute("data-row", &r.to_string()).unwrap();
            cell_div.set_attribute("data-col", &c.to_string()).unwrap();
            cell_div.set_text_content(Some(&cell_val.to_string()));
            row_div.append_child(&cell_div).unwrap();
        }
        container.append_child(&row_div).unwrap();
    }
}

fn update_pascal_sum(state: &AppState, sum_display: &Element) {
    let sum: BigInt = state
        .highlighted_cells
        .iter()
        .filter_map(|(r, c)| state.pascal_triangle.get(*r).and_then(|row| row.get(*c)))
        .sum();
    sum_display.set_text_content(Some(&sum.to_string()));
}

// --- Main App Logic ---
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let input_element = document
        .get_element_by_id("expression-input")
        .unwrap()
        .dyn_into::<HtmlTextAreaElement>()
        .unwrap();
    let calculate_button = document.get_element_by_id("calculate-button").unwrap();
    let result_output = document.get_element_by_id("result-output").unwrap();
    let history_list_element = document.get_element_by_id("history-list").unwrap();
    let eval_input = document
        .get_element_by_id("eval-input")
        .unwrap()
        .dyn_into::<HtmlInputElement>()
        .unwrap();
    let eval_button = document.get_element_by_id("eval-button").unwrap();
    let eval_result = document.get_element_by_id("eval-result").unwrap();
    let basis_selector = document
        .get_element_by_id("basis-selector")
        .unwrap()
        .dyn_into::<HtmlSelectElement>()
        .unwrap();

    // App State
    let app_state = Rc::new(RefCell::new(AppState {
        history: Vec::new(),
        current_poly: None,
        basis: Basis::Standard,
        pascal_rows: 7,
        pascal_triangle: generate_pascal_triangle(7),
        highlighted_cells: HashSet::new(),
    }));

    // --- Tab Switching Logic ---
    let tab_nav = document.query_selector(".tab-nav").unwrap().unwrap();
    let document_clone = document.clone();
    let on_tab_click = Closure::<dyn FnMut(_)>::new(move |event: Event| {
        let target = event.target().unwrap().dyn_into::<HtmlElement>().unwrap();
        if !target.class_list().contains("tab-button") {
            return;
        }

        let tab_id = target.id().replace("-btn", "");

        // Update button active state
        let nav = target.parent_element().unwrap();
        for i in 0..nav.children().length() {
            let child = nav.children().item(i).unwrap();
            child.class_list().remove_1("active").unwrap();
        }
        target.class_list().add_1("active").unwrap();

        // Update content active state
        let content_parent = document_clone
            .query_selector(".app-container")
            .unwrap()
            .unwrap();
        for i in 0..content_parent.children().length() {
            if let Some(child) = content_parent.children().item(i) {
                if child.class_list().contains("tab-content") {
                    child
                        .dyn_into::<HtmlElement>()
                        .unwrap()
                        .style()
                        .set_property("display", "none")
                        .unwrap();
                }
            }
        }
        document_clone
            .get_element_by_id(&tab_id)
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap()
            .style()
            .set_property("display", "block")
            .unwrap();
    });
    tab_nav
        .add_event_listener_with_callback("click", on_tab_click.as_ref().unchecked_ref())
        .unwrap();
    on_tab_click.forget();

    // --- Calculator Tab Logic ---
    // Basis Selector handler
    {
        let state_clone = Rc::clone(&app_state);
        let result_clone = result_output.clone();
        let history_list_clone = history_list_element.clone();
        let basis_selector_clone = basis_selector.clone();

        let on_basis_change = Closure::<dyn FnMut()>::new(move || {
            let mut state = state_clone.borrow_mut();
            let new_basis_str = basis_selector_clone.value();
            state.basis = if new_basis_str == "binomial" {
                Basis::Binomial
            } else {
                Basis::Standard
            };
            rerender_result(&state, &result_clone, &history_list_clone);
        });

        basis_selector
            .add_event_listener_with_callback("change", on_basis_change.as_ref().unchecked_ref())
            .unwrap();
        on_basis_change.forget();
    }

    // Main calculation handler
    {
        let state_clone = Rc::clone(&app_state);
        let input_clone = input_element.clone();
        let result_clone = result_output.clone();
        let history_list_clone = history_list_element.clone();

        let on_calc = Closure::<dyn FnMut()>::new(move || {
            perform_calculation(
                &input_clone,
                &result_clone,
                &mut state_clone.borrow_mut(),
                &history_list_clone,
            );
        });
        calculate_button
            .add_event_listener_with_callback("click", on_calc.as_ref().unchecked_ref())
            .unwrap();
        on_calc.forget();
    }

    // Ctrl+Enter handler for main input
    {
        let state_clone = Rc::clone(&app_state);
        let input_clone = input_element.clone();
        let result_clone = result_output.clone();
        let history_list_clone = history_list_element.clone();

        let on_keydown = Closure::<dyn FnMut(_)>::new(move |event: KeyboardEvent| {
            if event.key() == "Enter" && (event.ctrl_key() || event.meta_key()) {
                event.prevent_default();
                perform_calculation(
                    &input_clone,
                    &result_clone,
                    &mut state_clone.borrow_mut(),
                    &history_list_clone,
                );
            }
        });
        input_element
            .add_event_listener_with_callback("keydown", on_keydown.as_ref().unchecked_ref())
            .unwrap();
        on_keydown.forget();
    }

    // Evaluation button handler
    {
        let state_clone = Rc::clone(&app_state);
        let eval_input_clone = eval_input.clone();
        let eval_result_clone = eval_result.clone();

        let on_eval = Closure::<dyn FnMut()>::new(move || {
            perform_evaluation(&eval_input_clone, &eval_result_clone, &state_clone.borrow());
        });
        eval_button
            .add_event_listener_with_callback("click", on_eval.as_ref().unchecked_ref())
            .unwrap();
        on_eval.forget();
    }

    // Enter key handler for eval input
    {
        let state_clone = Rc::clone(&app_state);
        let eval_input_clone = eval_input.clone();
        let eval_result_clone = eval_result.clone();

        let on_keydown = Closure::<dyn FnMut(_)>::new(move |event: KeyboardEvent| {
            if event.key() == "Enter" {
                event.prevent_default();
                perform_evaluation(&eval_input_clone, &eval_result_clone, &state_clone.borrow());
            }
        });
        eval_input
            .add_event_listener_with_callback("keydown", on_keydown.as_ref().unchecked_ref())
            .unwrap();
        on_keydown.forget();
    }

    // --- Pascal's Triangle Tab Logic ---
    let pascal_container = document
        .get_element_by_id("pascal-triangle-container")
        .unwrap();
    let sum_display = document.get_element_by_id("highlight-sum-display").unwrap();
    let rows_input = document
        .get_element_by_id("pascal-rows-input")
        .unwrap()
        .dyn_into::<HtmlInputElement>()
        .unwrap();

    // Initial render
    render_pascal_triangle(&app_state.borrow(), &pascal_container);

    let on_rows_change = {
        let state_clone = Rc::clone(&app_state);
        let container_clone = pascal_container.clone();
        let sum_display_clone = sum_display.clone();
        let rows_input_clone = rows_input.clone();
        Closure::<dyn FnMut()>::new(move || {
            let mut state = state_clone.borrow_mut();
            let new_rows = rows_input_clone.value().parse().unwrap_or(7);
            state.pascal_rows = new_rows;
            state.pascal_triangle = generate_pascal_triangle(new_rows);
            state.highlighted_cells.clear();
            render_pascal_triangle(&state, &container_clone);
            update_pascal_sum(&state, &sum_display_clone);
        })
    };
    rows_input
        .add_event_listener_with_callback("change", on_rows_change.as_ref().unchecked_ref())
        .unwrap();
    on_rows_change.forget();

    let on_cell_click = {
        let state_clone = Rc::clone(&app_state);
        let container_clone = pascal_container.clone();
        let sum_display_clone = sum_display.clone();
        Closure::<dyn FnMut(_)>::new(move |event: Event| {
            let target = event.target().unwrap().dyn_into::<HtmlElement>().unwrap();
            let row: usize = target.dataset().get("row").unwrap().parse().unwrap();
            let col: usize = target.dataset().get("col").unwrap().parse().unwrap();

            let mut state = state_clone.borrow_mut();
            if state.highlighted_cells.contains(&(row, col)) {
                state.highlighted_cells.remove(&(row, col));
            } else {
                state.highlighted_cells.insert((row, col));
            }
            render_pascal_triangle(&state, &container_clone);
            update_pascal_sum(&state, &sum_display_clone);
        })
    };
    pascal_container
        .add_event_listener_with_callback("click", on_cell_click.as_ref().unchecked_ref())
        .unwrap();
    on_cell_click.forget();
}
