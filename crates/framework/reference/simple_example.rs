// use bevy::prelude::*;
// use prockit_framework::{
//     Bounds, GenerateAction, NodeCommands, ProceduralNode, Provider, Providing,
// };
//
// const SOURCE_NUMBER: Names = Names::new("source number").with_alt("number");
//
// #[derive(Component, Default, Reflect)]
// struct DecreasingNumber {
//     my_number: f32,
// }
//
// impl DecreasingNumber {
//     fn source_number(&self) -> f32 {
//         self.my_number
//     }
// }
//
// impl ProceduralNode for DecreasingNumber {
//     fn enhance(&self, bounds: &Bounds, _provider: &Provider, mut node_commands: NodeCommands) {
//         node_commands.add_child::<Self>(bounds);
//         node_commands.add_child::<Self>(bounds);
//     }
//
//     fn generate(&mut self, _bounds: &Bounds, provider: &Provider) -> GenerateAction {
//         let test = provider.get::<(f32, f32), f32>(
//             [Caller::from("first"), Caller::from("second")],
//             Caller::from("add"),
//         );
//         let test_2 = test((1.0, 2.0));
//         if let Some(source_number) = provider.get::<(), f32>(&[], SOURCE_NUMBER.as_caller()) {
//             self.my_number = source_number(()) - 1.0;
//         } else {
//             self.my_number = 10.0
//         }
//         GenerateAction::RegenerateChildren
//     }
//
//     fn init() -> Self {
//         Self::default()
//     }
//
//     fn providing(&self) -> Providing {
//         Providing::new().with(SOURCE_NUMBER, Self::source_number)
//     }
//
//     fn view(&self, _bounds: &Bounds, _distance: f32, _time: f32) -> DetailAction {
//         let error = 1e-4;
//         if self.my_number > 0.0 + error {
//             DetailAction::Enhance
//         } else if self.my_number < 0.0 - error {
//             DetailAction::Compress
//         } else {
//             DetailAction::None
//         }
//     }
// }
//
// fn main() {
//     // App::new()
//     //     .add_plugins(DefaultPlugins)
//     //     .add_plugins(ProckelPlugin::<SimpleProckel>::new())
//     //     .add_systems(Startup, setup)
//     //     .run();
// }
//
// // struct Empty;
// //
// // fn setup(mut commands: Commands) {
// //     commands.spawn(SimpleProckel::default());
// //     let mut provider = Provider::default();
// //     provider.register("Source Number", |empty: ()| 0.0);
// // }
//

fn main() {
    println!("WIP!");
}
