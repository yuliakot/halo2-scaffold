use clap::Parser;
use halo2_base::gates::{GateInstructions, RangeChip, RangeInstructions};
use halo2_base::utils::ScalarField;
use halo2_base::{AssignedValue, Context};
use halo2_scaffold::scaffold::cmd::Cli;
use halo2_scaffold::scaffold::run;
use serde::{Deserialize, Serialize};
use halo2_base::QuantumCell::Witness;

use std::env::var;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CircuitInput {
    pub x: String, 
}

fn some_algorithm_in_zk<F: ScalarField>(
    ctx: &mut Context<F>,
    input: CircuitInput,
    make_public: &mut Vec<AssignedValue<F>>,
) 
    {
    let x = u64::from_str_radix(&input.x, 10).expect("deserialize field element should not fail");
    assert!( x < 1024*64);
    let y = x/32;
    
    let [x, y] = [x, y].map(F::from);


    let lookup_bits =
        var("LOOKUP_BITS").unwrap_or_else(|_| panic!("LOOKUP_BITS not set")).parse().unwrap();

        
     let x = ctx.load_witness(x);
     let y = ctx.load_witness(y);
     make_public.push(x);
     make_public.push(y);

    let range = RangeChip::default(lookup_bits);


    let z = range.gate().mul_add(ctx, y, Witness(F::from(32).neg()), x);
    range.check_less_than(ctx, z, Witness(F::from(32)), lookup_bits);
    range.check_less_than(ctx, Witness(F::zero()), z, lookup_bits);

    println!("\nOut : {:?}\n", y.value());
    //z
}

fn main() {
    env_logger::init();

    let args = Cli::parse();

    // run different zk commands based on the command line arguments
    run(some_algorithm_in_zk, args);
}
