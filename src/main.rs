use yew::prelude::*;

#[derive(Debug)]
pub struct Model;
impl Component for Model {
    type Message = ();
    type Properties = ();

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        unimplemented!()
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <main>
                <h1>{ "Let's cut the link!" }</h1>
                <span class="subtitle">{ "from Yew with " }<i class="heart" /></span>
            </main>
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}
