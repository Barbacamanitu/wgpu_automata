use wgpu_automata::run;

fn main() {
    pollster::block_on(run());
    //wgpu_automata::test();
}
