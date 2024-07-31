mod packet;

use packet::{Packet::{self, *}, Message};
use std::collections::VecDeque;


/// 뭉쳐온 패킷 분리 및 잘린 패킷 이어붙이기를 수행하는 큐 형태의 Parser
pub struct PacketParser(VecDeque<Packet>);

impl PacketParser {
    pub fn new() -> Self {
        Self(VecDeque::new())
    }

    pub fn push(&mut self, data: &[u8]) {
        if data.len() == 0 {
            return;
        }

        // trim하면 안됨
        // 'init 3 1 2'이 들어올때 'init 3', ' 1 2',으로 나눠져서 오면
        // 'init 31 2'가 되어버림
        let mut data = data.split(|&x| x == b'\n')
            .map(|x| x.to_vec())
            .collect::<Vec<_>>();

        if let Some(Incomplete(prev)) = self.0.back() {
            data[0] = prev.iter().chain(data[0].iter())
                .copied()
                .collect();
            self.0.pop_back();
        }

        for data in data {
            if data.is_empty() {
                continue;
            }

            self.0.push_back(Self::parse(data));
        }
    }

    fn parse(data: Vec<u8>) -> Packet {
        let message: Vec<&[u8]> = data.split(|&x| x == b' ')
            .collect();

        match message[0] {
            b"update" => {
                Complete(Message::Update)
            },

            b"remove" => {
                Complete(Message::Remove)
            },

            _ => Incomplete(data),
        }
    }

    /// 한개 남았을 때 Incomplete이면 아직 완성 안된것이므로 pop하지 않음.  
    /// 두개 이상 남았을때 제일 앞 패킷이 Incomplete이면 모종의 이유로 완성 안된것이므로 값을 버리기 위해 pop.  
    pub fn pop(&mut self) -> Option<Packet> {
        if self.0.len() == 1 {
            if let Some(Incomplete(_)) = self.0.front() {
                return None;
            }
        }

        self.0.pop_front()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> std::collections::vec_deque::Iter<Packet> {
        self.0.iter()
    }
}





#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut parser = PacketParser::new();

        parser.push(b"update\nremove\ninit\n");
        
        let answer = vec![
            Complete(Message::Update),
            Complete(Message::Remove),
            Incomplete(b"init".to_vec()),
        ];

        assert_eq!(parser.len(), answer.len());
        
        for (a, b) in parser.iter().zip(answer.iter()) {
            assert_eq!(a, b);
        }

        assert_eq!(parser.pop(), Some(Complete(Message::Update)));

        parser.push(b"upda");
        assert_eq!(parser.iter().last(), Some(&Incomplete(b"upda".to_vec())));

        parser.push(b"te\n");
        assert_eq!(parser.iter().last(), Some(&Complete(Message::Update)));


    }
}
