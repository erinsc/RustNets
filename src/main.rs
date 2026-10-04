mod sparsevec;
mod interactionnet;
use interactionnet::*;

fn main() -> Result<(), NetError> {
    let mut net = InteractionNet::<(), ()>::new();
    let add = net.register_symbol("Add", 2, ());
    let z = net.register_symbol("Z", 0, ());
    let s = net.register_symbol("S", 1, ());

    let mut book = RuleBook::new();

    let peano_zero = rule![
        [add, 0, 0],
        [z]
    ];
    let peano_add = rule![
        [add, [s, 0], 1], 
        [s, [add, 0, 1]]
    ];

    println!("{:?}", peano_zero);
    println!("{:?}", peano_add);

    book.register_rule(peano_zero);
    book.register_rule(peano_add);
    
    let an = net.create_node(add, ())?;
    let sn = net.create_node(s, ())?;
    let zn = net.create_node(z, ())?;
    let zn2 = net.create_node(z, ())?;

    net.link(port![an, 0], port![sn, 0])?;
    net.link(port![sn, 1], port![zn, 0])?;
    net.link(port![an, 2], port![zn2, 0])?;

    println!("{}", net);

    net.interact(an, sn, &book)?;

    println!("{}", net);

    net.interact(2, 5, &book)?;

    println!("{}", net);

    Ok(())
}
