use collation::collation;
use message;
use client_thread;

use ethereum_types;

use std::thread;
use std::sync::mpsc;
use std::collections::HashMap;

pub struct Notary {
    id: ethereum_types::U256,
    selected: bool,
    shard_id: ethereum_types::U256,
    collation_vectors: HashMap<ethereum_types::U256, Vec<collation::Collation>>,
    proposal_vectors: HashMap<ethereum_types::U256, Vec<collation::Collation>>,
    smc_listener: mpsc::Receiver<message::Message>,
    manager_listener: mpsc::Receiver<client_thread::Command>
}

impl Notary {
    /// Creates a new Notary
    /// 
    /// #Inputs
    /// 
    /// smc_listener: mpsc::Receiver<message::Message>
    /// manager_listener: mpsc::Receiver<client_thread::Command>
    /// 
    /// The smc_listener allows the Notary to receive messages from the SMC Listener, and the manager_listener allows the thread to receive commands from outside the thread.
    pub fn new(smc_listener: mpsc::Receiver<message::Message>, manager_listener: mpsc::Receiver<client_thread::Command>) -> Notary {
        Notary {
            id: ethereum_types::U256::from_dec_str("0").unwrap(),
            selected: false,
            shard_id: ethereum_types::U256::from_dec_str("0").unwrap(),
            collation_vectors: HashMap::new(),
            proposal_vectors: HashMap::new(),
            smc_listener,
            manager_listener
        }
    }

    /// 
    pub fn run(&mut self) {
        loop {
            // Asynchronously get message from the thread manager
            let manager_msg = self.manager_listener.try_iter().next();

            // Respond to the thread manager message
            match manager_msg {
                Some(msg) => {
                    println!("Received pending message {:?} in thread {:?} from another thread", msg, thread::current());
                    match msg {
                        client_thread::Command::Terminate => { break }
                    }
                },
                None => {
                    // println!("No more pending messages from other threads to thread {:?} or channel hung up", thread::current());
                }
            }

            // Asynchronously get message from the SMC listener
            let smc_msg = self.smc_listener.try_iter().next();

            // Respond to SMC listener message
            match smc_msg {
                Some(msg) => {
                    println!("Received pending message {:?} in thread {:?} from SMC Listener", msg, thread::current());
                    match msg {
                        message::Message::Selected { value } => { self.selected = value; }
                        message::Message::ShardId { value } => { self.shard_id = value; },
                        message::Message::Collation { value } => { self.store_collation(value); },
                        message::Message::Proposal { value } => { self.store_proposal(value); }
                    }
                },
                None => {
                    // println!("No more pending messages from SMC Listener to thread {:?} or channel hung", thread::current());
                }
            }

            if self.selected {
                self.get_availability();
                self.submit_vote();
            }
        }
    }

    fn store_collation(&mut self, collation: collation::Collation) {
        println!("Storing in notary id {} a new collation mapped to shard id {}", self.id, self.shard_id);
        self.collation_vectors.entry(self.shard_id).or_insert(vec![]);
        let vector = self.collation_vectors.get_mut(&self.shard_id).unwrap();
        vector.push(collation);
    }

    fn store_proposal(&mut self, proposal: collation::Collation) {
        println!("Storing in notary id {} a new proposal collation mapped to shard id {}", self.id, self.shard_id);
        self.proposal_vectors.entry(self.shard_id).or_insert(vec![]);
        let vector = self.proposal_vectors.get_mut(&self.shard_id).unwrap();
        vector.push(proposal);
    }

    fn get_availability(&mut self) {}

    fn submit_vote(&self) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use collation::header;
    use collation::body;

    fn generate_genesis_collation() -> collation::Collation {
        let g_shard_id = ethereum_types::U256::from_dec_str("0").unwrap();
        let g_chunk_root = ethereum_types::H256::zero();
        let g_period = ethereum_types::U256::from_dec_str("0").unwrap();
        let g_proposer_address = ethereum_types::Address::zero();
        let genesis_header = header::Header::new(g_shard_id, g_chunk_root, g_period, g_proposer_address);
        collation::Collation::new(genesis_header, body::Body)
    }

    fn generate_collation() -> collation::Collation {
        let shard_id = ethereum_types::U256::from_dec_str("0").unwrap();
        let chunk_root = ethereum_types::H256::zero();
        let period = ethereum_types::U256::from_dec_str("0").unwrap();
        let proposer_address = ethereum_types::Address::zero();
        let collation_header = header::Header::new(shard_id, chunk_root, period, proposer_address);
        collation::Collation::new(collation_header, body::Body)
    }

    fn generate_notary() -> Notary {
        let (_tx, rx) = mpsc::channel();
        let (_mtx, mrx) = mpsc::channel();
        Notary::new(rx, mrx)
    }

    #[test]
    fn it_stores_collation() {
        let mut notary = generate_notary();

        // Genesis collation
        let genesis_collation = generate_genesis_collation();
        let genesis_collation_cmp = genesis_collation.clone();

        // First collation
        let first_collation = generate_collation();
        let first_collation_cmp = first_collation.clone();

        // Push genesis collation into notary
        notary.store_collation(genesis_collation);
        notary.store_collation(first_collation);

        // Check that the operations succeded
        let vector = notary.collation_vectors.get(&ethereum_types::U256::from_dec_str("0").unwrap()).unwrap();
        assert_eq!(vector[0], genesis_collation_cmp);
        assert_eq!(vector[1], first_collation_cmp);
    }

    #[test]
    fn it_stores_proposals() {
        let mut notary = generate_notary();

        // Generate proposal
        let proposal = generate_collation();
        let proposal_cmp = proposal.clone();

        // Store proposal in notary
        notary.store_proposal(proposal);

        // Check that the operations succeeded
        let vector = notary.proposal_vectors.get(&ethereum_types::U256::from_dec_str("0").unwrap()).unwrap();
        assert_eq!(vector[0], proposal_cmp);
    }

    #[test]
    #[ignore]
    fn it_selects_vote() {
        assert!(false);
    }

    #[test]
    #[ignore]
    fn it_submits_vote() {
        assert!(false);
    }
}
