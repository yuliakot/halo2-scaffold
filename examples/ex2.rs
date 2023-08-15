use clap::Parser;
use halo2_base::gates::{GateInstructions, RangeChip, RangeInstructions, GateChip};
use halo2_base::utils::ScalarField;
use halo2_base::{AssignedValue, Context};
use halo2_scaffold::scaffold::cmd::Cli;
use halo2_scaffold::scaffold::run;
use serde::{Deserialize, Serialize};
use halo2_base::QuantumCell::{Witness, Constant};
use ark_std::fs::File;

use std::env::var;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CircuitInput {
    pub start: String, // field element, but easier to deserialize as a string
    pub end: String, // field element, but easier to deserialize as a string
    pub arr: Vec<u64>, // array
}

// public inputs:
// * An array `arr` of length 1000
// * `start`, an index guaranteed to be in `[0, 1000)`
// * `end`, an index guaranteed to be in `[0, 1000)`
// * It is also known that `start <= end`

// public outputs:
// * An array `out` of length 1000 such that
//   * the first `end - start` entries of `out` are the subarray `arr[start:end]`
//   * all other entries of `out` are 0.


fn some_algorithm_in_zk<F: ScalarField>(
    ctx: &mut Context<F>,
    input: CircuitInput,
    make_public: &mut Vec<AssignedValue<F>>,
) //-> impl FnOnce(&mut Context<F>) + Clone  {
    {
    //let lookup_bits =
    //    var("LOOKUP_BITS").unwrap_or_else(|_| panic!("LOOKUP_BITS not set")).parse().unwrap();

    const N: usize = 1000;
    
    let start = u64::from_str_radix(&input.start, 10).expect("deserialize field element should not fail");
    let end = u64::from_str_radix(&input.end, 10).expect("deserialize field element should not fail");
    let arr: Vec::<u64> = input.arr;

    assert_eq!(arr.len(), N);
        //let m = arr.map(|x| (x as f64).log2().round() as u32).into_iter().max().unwrap() + 1;
    let arr: Vec::<F> = arr.iter().map(|x|F::from(*x)).collect();
    

    let chip: GateChip<F> = GateChip::default();

    let mut arr: [AssignedValue<F>; N] = ctx.assign_witnesses(arr).try_into().unwrap();
    let [startt, endd]: [AssignedValue<F>; 2] = ctx.assign_witnesses([F::from(start), F::from(end)]).try_into().unwrap();

    make_public.push(startt);
    make_public.push(endd);
    make_public.extend(&arr);

    //let m = ctx.load_witness(F::from(m as u64));

    //for N rounds: 
    //  for arr[i] in arr:
    //      new_arr[i] = arr[i - 1]*(start is not zero) + arr[i]*(start is zero)
    //  new_start = start + (-1)*(start is zero)
    //  new_end = end + (-1)*(start is zero)
    
    let mut start_is_zero: Vec::<AssignedValue::<F>> = vec![];
    let mut start_is_non_zero: Vec::<AssignedValue::<F>> = vec![];
    let mut start: Vec::<AssignedValue::<F>>= vec![];
    let mut pow_two: Vec::<AssignedValue::<F>>= vec![];
    let mut end: Vec::<AssignedValue::<F>>= vec![];
    

    start.push(startt);
    end.push(endd);

    let mut put : Vec<Vec<AssignedValue<F>>> = vec![];
    for i in 0..N{
        put.push(vec![arr[i]]);
    }
    let mut out: [Vec<AssignedValue<F>>; N] = put.try_into().unwrap();

    let mut maybe_ai: Vec::<AssignedValue::<F>> = vec![];
    let mut maybe_ai_plus_one: Vec::<AssignedValue::<F>> = vec![];    


    let logn = (N as f64).log(2.0) as usize;
    // this is bits, but right to left...
    let mut start_bits = chip.num_to_bits(ctx, startt, logn + 1);
    let mut curr = 2_i32.pow(logn as u32) as usize;


    for _ in 0..logn{
        start_is_zero.push(chip.is_zero(ctx, *start_bits.last().unwrap()));
        start_is_non_zero.push(chip.sub(ctx, Constant(F::one()), *start_is_zero.last().unwrap()));
        for i in 0..(N - curr){
            maybe_ai.push(chip.mul(ctx, *out[i].last().unwrap(), *start_is_zero.last().unwrap()));
            maybe_ai_plus_one.push(chip.mul(ctx, *out[i+ curr].last().unwrap(), *start_is_non_zero.last().unwrap()));
            out[i].push(chip.add(ctx,* maybe_ai.last().unwrap(), *maybe_ai_plus_one.last().unwrap()));
        }
        pow_two.push(ctx.load_constant(-F::from(curr as u64)));
        
        start.push(chip.mul_add(ctx,  *pow_two.last().unwrap(), *start_bits.last().unwrap(), *start.last().unwrap()));
        end.push(chip.mul_add(ctx,  *pow_two.last().unwrap(), *start_bits.last().unwrap(), *end.last().unwrap()));
        curr /= 2;
        start_bits.pop();
    }

    let mut selector: Vec::<AssignedValue::<F>> = vec![];
    selector.push(ctx.load_witness(F::one()));

    
    let mut the_end: Vec::<AssignedValue::<F>> = vec![];
    let mut ii: Vec::<AssignedValue::<F>> = vec![];

    for i in 0..(N-1){
        ii.push(ctx.load_constant(F::from(i as u64)));
        the_end.push(chip.is_equal(ctx, *ii.last().unwrap(), *end.last().unwrap()));  
        selector.push(chip.sub(ctx, *selector.last().unwrap(), *the_end.last().unwrap()));
        out[i].push(chip.mul(ctx, *out[i].last().unwrap(), *selector.last().unwrap()));
    }

    let ans = out.map(|x| *x.last().unwrap());
    make_public.extend(ans);

}

fn main() {
    env_logger::init();

    let args = Cli::parse();

    run(some_algorithm_in_zk, args);
}
