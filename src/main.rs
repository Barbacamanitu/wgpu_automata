use wgpu_automata::run;

fn main() {
    //pollster::block_on(run());
    pollster::block_on(wgpu_automata::test_compute());
}
