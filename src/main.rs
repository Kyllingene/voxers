use voxers::run;

fn main() -> anyhow::Result<()> {
    pollster::block_on(run())
}
