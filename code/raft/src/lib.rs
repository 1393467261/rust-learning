use std::{fmt::Display, time::{Instant, Duration}, sync::{Arc, Mutex, mpsc::{self, Sender, Receiver}}, thread, collections::{self, HashMap}, hash::Hash};
use rand::{thread_rng, Rng};

enum State {
    Follower,
    Leader,
    Candidate,
    Dead,
}

enum Message {
    ResetTimer,
    BecomeFollower(i32),
    NewCommitReady,
}

impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            State::Follower => write!(f, "Follower"),
            State::Leader => write!(f, "Leader"),
            State::Candidate => write!(f, "Candidate"),
            State::Dead => write!(f, "Dead"),
        }
    }
}

struct ConsensusModule {
    id: i32,
    peer_ids: Vec<i32>,
    server: Server,
    // persisent
    current_term: i32,
    voted_for: i32,
    log: Vec<LogEntry>,
    // volatile
    commit_index: i32,
    last_applied: i32,
    state: State,
    election_reset_event: Instant,
    // volatile in leader
    next_index: HashMap<i32, i32>,
    match_index: HashMap<i32, i32>,

    msg_sender: Sender<Message>,
    msg_receiver: Receiver<Message>,
}

impl ConsensusModule {
    fn new(id: i32, peer_ids: Vec<i32>, server: Server) -> ConsensusModule {
        let (sender, receiver) = mpsc::channel();

        ConsensusModule { 
            id, 
            peer_ids, 
            server, 
            current_term: 0, 
            voted_for: -1, 
            log: vec![], 
            state: State::Follower, 
            election_reset_event: Instant::now(),
            msg_sender: sender,
            msg_receiver: receiver,
            commit_index: 0,
            last_applied: 0,
            next_index: HashMap::new(),
            match_index: HashMap::new(),
        }
    }

    fn run_election_timer(cm: Arc<Mutex<ConsensusModule>>) {
        let sender = cm.lock().unwrap().msg_sender.clone();
        let term_started = cm.lock().unwrap().current_term;

        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_millis(10));

                match cm.lock().unwrap().state {
                    State::Follower | State::Candidate => {},
                    _ => break,
                }

                if term_started != cm.lock().unwrap().current_term {
                    break;
                }

                if Instant::now().duration_since(cm.lock().unwrap().election_reset_event).as_millis() as u64 > 
                    ConsensusModule::election_timeout() {
                    ConsensusModule::start_election(Arc::clone(&cm));
                }
                
                sender.send(Message::ResetTimer).unwrap();
            }
        });
    }

    fn run_message_handler(cm: Arc<Mutex<ConsensusModule>>) {
        thread::spawn(move || {
            while let Ok(msg) = cm.lock().unwrap().msg_receiver.recv() {
                match msg {
                    Message::ResetTimer => {
                        cm.lock().unwrap().reset_election_event();
                    }
                    Message::BecomeFollower(term) => {
                        cm.lock().unwrap().become_follower(term);
                    }
                    Message::NewCommitReady => {
                        todo!();
                    }
                }
            }
        });
    }

    fn election_timeout() -> u64 {
        thread_rng().gen_range(0..150) + 150
    }

    fn reset_election_event(&mut self) {
        self.election_reset_event = Instant::now();
    }

    fn become_follower(&mut self, term: i32) {
        self.state = State::Follower;
        self.current_term = term;
        self.voted_for = -1;
        self.reset_election_event();
    }

    fn report(&self) -> (i32, i32, bool) {
        (self.id, self.current_term, matches!(self.state, State::Leader))
    }

    fn submit(&mut self, command: ()) -> bool {
        if matches!(self.state, State::Leader) {
            self.log.push(LogEntry { command, term: self.current_term });
            true
        } else {
            false
        }
    }

    fn stop(&self) {}

    fn last_log_index_and_term(&self) -> (i32, i32) {
        if self.log.len() > 0 {
            let last_index = self.log.len() - 1;
            (last_index as i32, self.log[last_index].term)
        } else {
            (-1, -1)
        }
    }

    fn request_vote(&mut self, args: RequestVoteArgs) -> RequestVoteReply {
        if args.term > self.current_term {
            self.become_follower(args.term);
        }

        let mut reply = RequestVoteReply {
            term: self.current_term,
            vote_granted: false,
        };

        let (last_log_index, last_log_term) = self.last_log_index_and_term();

        if args.term == self.current_term && 
            (self.voted_for == -1 || self.voted_for == args.candidate_id) &&
                (args.last_log_term > last_log_term ||
                    (args.last_log_term == last_log_term && args.last_log_index >= last_log_index)) {
            reply.vote_granted = true;
            self.voted_for = args.candidate_id;
            self.reset_election_event();
        } 

        reply
    }

    fn append_entries(&mut self, mut args: AppendEntriesArgs) -> AppendEntriesReply {
        if args.term > self.current_term {
            self.become_follower(args.term);
        };

        let mut reply = AppendEntriesReply {
            term: self.current_term,
            success: false,
        };

        if args.term == self.current_term {
            if matches!(self.state, State::Follower) {
                self.become_follower(args.term);
            }

            self.reset_election_event();

            if args.prev_log_index == -1 ||
                (args.prev_log_index < self.log.len() as i32 && args.prev_log_term == self.log[args.prev_log_index as usize].term) {
                reply.success = true;

                let mut log_insert_index = args.prev_log_index + 1;
                let mut new_entries_index = 0;

                loop {
                    if log_insert_index as usize >= self.log.len() || new_entries_index >= args.entries.len() {
                        break;
                    }
                    
                    if self.log[log_insert_index as usize].term != args.entries[new_entries_index].term {
                        break;
                    }

                    log_insert_index += 1;
                    new_entries_index += 1;
                }

                if new_entries_index < args.entries.len() {
                    // cm.log = append(cm.log[:logInsertIndex], args.Entries[newEntriesIndex:]...)
                    for _ in log_insert_index as usize..self.log.len() {
                        self.log.pop();
                    }
                    for i in new_entries_index..args.entries.len() {
                        self.log.push(args.entries.remove(i));
                    }
                }

                if args.leader_commit > self.commit_index {
                    self.commit_index = std::cmp::min(args.leader_commit, self.log.len() as i32);
                    self.msg_sender.send(Message::NewCommitReady).unwrap();
                }
            }
        }

        reply
    }

    fn start_election(cm: Arc<Mutex<ConsensusModule>>) {
        let term_started;
        let mut vote_received_counter = Arc::new(Mutex::new(1));
        let id;
        let peers_count;

        let mut guard = cm.lock().unwrap();
       
        guard.current_term += 1;
        term_started = guard.current_term;
        id = guard.id;
        peers_count = guard.peer_ids.len();
        guard.reset_election_event();
        guard.voted_for = guard.id;
        
        drop(guard);

        for peer_id in cm.lock().unwrap().peer_ids.clone() {
            let cm = Arc::clone(&cm);
            let vote_received_counter = Arc::clone(&vote_received_counter);

            thread::spawn(move || {
                let (last_log_index, last_log_term) = cm.lock().unwrap().last_log_index_and_term();

                let args = RequestVoteArgs {
                    term: term_started,
                    candidate_id: id,
                    last_log_index,
                    last_log_term,
                };

                let reply = cm.lock().unwrap().server.request_vote(peer_id, args);
                
                if matches!(cm.lock().unwrap().state, State::Candidate) {
                    return;
                }

                if reply.term > term_started {
                    cm.lock().unwrap().become_follower(reply.term);
                    return;
                } else if reply.term == term_started {
                    *vote_received_counter.lock().unwrap() += 1;
                    if *vote_received_counter.lock().unwrap() * 2 > peers_count + 1 {
                        ConsensusModule::start_leader(Arc::clone(&cm));
                        return;
                    }
                }
            });
        }

        ConsensusModule::run_election_timer(Arc::clone(&cm));
    }

    fn start_leader(cm: Arc<Mutex<ConsensusModule>>) {
        let mut guard = cm.lock().unwrap();

        let log_len = guard.log.len() as i32;
        guard.state = State::Leader;
        for peer_id in guard.peer_ids.clone() {
            guard.next_index.insert(peer_id, log_len);
            guard.match_index.insert(peer_id, -1);
        }

        drop(guard);

        thread::spawn(move || {
            loop {
                // todo why send heartbeat asynchronously?
                ConsensusModule::send_heartbeat(Arc::clone(&cm));

                thread::sleep(Duration::from_millis(50));

                if matches!(cm.lock().unwrap().state, State::Leader) {
                    return;
                }
            }
        });
    }

    fn send_heartbeat(cm: Arc<Mutex<ConsensusModule>>) {
        let guard = cm.lock().unwrap();

        let term_started = guard.current_term;
        let log_len = guard.log.len();
        let id = guard.id;
        let leader_commit = guard.commit_index;

        drop(guard);

        for peer_id in cm.lock().unwrap().peer_ids.clone() {
            let prev_log_index;

            match cm.lock().unwrap().next_index.get(&peer_id) {
                Some(val) => {
                    prev_log_index = *val  - 1;
                },
                None => continue,
            }

            let mut prev_log_term = -1;
            if prev_log_index > 0 {
                if let Some(term) = cm.lock().unwrap().log.get(prev_log_index as usize) {
                    prev_log_term = term.term;
                }
            }

            let mut entries = Vec::new();
            for i in 1 + prev_log_index as usize..log_len {
                entries.push(cm.lock().unwrap().log[i].clone());
            }
            let entries_len = entries.len();

            let args = AppendEntriesArgs {
                term: term_started,
                leader_id: id,
                prev_log_index,
                prev_log_term,
                entries,
                leader_commit: leader_commit,
            };

            let cm = Arc::clone(&cm);
            
            thread::spawn(move || {
                let reply = cm.lock().unwrap().server.append_entries(peer_id, args);
                
                if reply.term > term_started {
                    cm.lock().unwrap().become_follower(reply.term);
                    // todo return where?
                    return;
                }

                if reply.term == term_started && matches!(cm.lock().unwrap().state, State::Leader) {
                    if reply.success {
                        if let Some(next_index) = cm.lock().unwrap().next_index.get_mut(&peer_id) {
                            *next_index = prev_log_index + 1 + entries_len as i32;
                        }

                        if let Some(match_index) = cm.lock().unwrap().match_index.get_mut(&peer_id) {
                            *match_index = prev_log_index + entries_len as i32;
                        }

                        let saved_commit_index = cm.lock().unwrap().commit_index;

                        for i in 1 + saved_commit_index as usize..log_len {
                            let mut guard = cm.lock().unwrap();
                            if let Some(log_entry) = guard.log.get(i) {
                                if log_entry.term == term_started {
                                    let mut match_count = 1;
                                    
                                    for peer_id in &guard.peer_ids {
                                        if let Some(match_index) = guard.match_index.get(peer_id) {
                                            if *match_index as usize >= i {
                                                match_count += 1;
                                            }
                                        }
                                    }

                                    if match_count * 2 > guard.peer_ids.len() + 1 {
                                        guard.commit_index += 1;
                                    }
                                }
                            }
                            drop(guard);
                        }

                        if saved_commit_index != cm.lock().unwrap().commit_index {
                            cm.lock().unwrap().msg_sender.send(Message::NewCommitReady).unwrap();
                        }
                    } else {
                        if let Some(next_index) = cm.lock().unwrap().next_index.get_mut(&peer_id) {
                            *next_index = prev_log_index;
                        }
                    }
                }
            });
        }
    }
}

struct RequestVoteArgs {
	term: i32,
	candidate_id: i32,
    last_log_index: i32,
    last_log_term: i32,
}

struct RequestVoteReply {
	term: i32,
	vote_granted: bool,
}

struct AppendEntriesArgs {
	term: i32,
	leader_id: i32,
	prev_log_index: i32,
    prev_log_term: i32,
	entries: Vec<LogEntry>,
	leader_commit: i32,
}

struct AppendEntriesReply {
    term: i32,
    success: bool,
}

struct Server {}

impl Server {
    fn request_vote(&self, peer_id: i32, args: RequestVoteArgs) -> RequestVoteReply {
        RequestVoteReply { term: -1, vote_granted: false }
    }

    fn append_entries(&self, peer_id: i32, args: AppendEntriesArgs) -> AppendEntriesReply {
        AppendEntriesReply { term: -1, success: false }
    }
}

#[derive(Debug, Copy, Clone)]
struct LogEntry {
    command: (),
    term: i32,
}

#[test]
fn test() {
    println!("compile success!")
}