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
        let mut data = data.split_inclusive(|&x| x == b'\n')
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
        let len = data.len();

        if data[len - 1] != b'\n' {
            return Incomplete(data);
        }

        let data = &data[..len - 1];

        let message: Vec<&[u8]> = data.split(|&x| x == b' ')
            .collect();

        match message[0] {
            b"update" => {
                Complete(Message::Update)
            },

            b"remove" => {
                Complete(Message::Remove)
            },

            _ => Complete(Message::Unknown),
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

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn front(&self) -> Option<&Packet> {
        self.0.front()
    }

    pub fn back(&self) -> Option<&Packet> {
        self.0.back()
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
    fn test_parse() {
        let mut parser = PacketParser::new();

        parser.push(b"update\n");
        assert_eq!(parser.pop(), Some(Complete(Message::Update)));

        parser.push(b"remove\n");
        assert_eq!(parser.pop(), Some(Complete(Message::Remove)));

        parser.push(b"init 3 2 5 6\n");
        assert_eq!(parser.pop(), Some(Complete(Message::Unknown)));
    }

    #[test]
    fn test_sliced_packet() {
        let mut parser = PacketParser::new();
        
        parser.push(b"upda");
        assert_eq!(parser.iter().last(), Some(&Incomplete(b"upda".to_vec())));
        assert_eq!(parser.pop(), None);

        parser.push(b"te\n");
        assert_eq!(parser.iter().last(), Some(&Complete(Message::Update)));
        assert_eq!(parser.pop(), Some(Complete(Message::Update)));

        parser.push(b"remove");
        assert_eq!(parser.iter().last(), Some(&Incomplete(b"remove".to_vec())));
        assert_eq!(parser.pop(), None);

        parser.push(b"\n");
        assert_eq!(parser.iter().last(), Some(&Complete(Message::Remove)));
        assert_eq!(parser.pop(), Some(Complete(Message::Remove)));

        parser.push(b"init 3 2 5 6\nupdate\nre");
        let quess = vec![
            Complete(Message::Unknown),
            Complete(Message::Update),
            Incomplete(b"re".to_vec()),
        ];
        for it in parser.iter().zip(quess.iter()) {
            assert_eq!(it.0, it.1);
        }

        parser.push(b"move\n");
        let quess = vec![
            Complete(Message::Unknown),
            Complete(Message::Update),
            Complete(Message::Remove),
        ];
        for it in parser.iter().zip(quess.iter()) {
            assert_eq!(it.0, it.1);
        }
    }

    #[test]
    fn test_empty_packet() {
        let mut parser = PacketParser::new();

        parser.push(b"");
        assert_eq!(parser.len(), 0);

        parser.push(b"\n");
        assert_eq!(parser.len(), 1);
        assert_eq!(parser.pop(), Some(Complete(Message::Unknown)));
    }

    #[test]
    fn test_incomplete_pop() {
        let mut parser = PacketParser::new();

        parser.push(b"upda");
        assert_eq!(parser.pop(), None);

        parser.push(b"te\n");
        assert_eq!(parser.pop(), Some(Complete(Message::Update)));
    }

    #[test]
    fn test_merged_packet() {
        let mut parser = PacketParser::new();
        
        // \n\n\n같은게 들어올 확률보다 끝에 \n하나만 붙은게 들어올 확률이 높음
        // -> \n\n\n같은걸 무시하게 하는 로직을 통과시키는게 낭비일 수 있음
        // -> 정상적인 패킷으로 취급
        let packets = b"update\nupdate\nremove\ninit 3 2 5 6\nhello!\n\n\n \nupdate\nstart localhost:8080\nremove\n";
        parser.push(packets);
        assert_eq!(parser.len(), packets.iter().filter(|&&x| x == b'\n').count());

        let quess = vec![
            Complete(Message::Update),
            Complete(Message::Update),
            Complete(Message::Remove),
            Complete(Message::Unknown),
            Complete(Message::Unknown),
            Complete(Message::Unknown),
            Complete(Message::Unknown),
            Complete(Message::Unknown),
            Complete(Message::Update),
            Complete(Message::Unknown),
            Complete(Message::Remove),
        ];
        for it in parser.iter().zip(quess.iter()) {
            assert_eq!(it.0, it.1);
        }
    }

    #[test]
    fn test_merged_and_sliced_packet() {
        let mut parser = PacketParser::new();
        
        let packets1 = b"update\nupdate";
        let packets2 = b"\nremove\ninit 3 2 5 ";
        let packets3 = b"6\nhello!\n\n\n \nupdate\nstart localh";
        let packets4 = b"ost:8080\nremove\n";
        parser.push(packets1);
        parser.push(packets2);
        parser.push(packets3);
        parser.push(packets4);
        assert_eq!(
            parser.len(), 
            packets1.iter().filter(|&&x| x == b'\n').count() + 
            packets2.iter().filter(|&&x| x == b'\n').count() + 
            packets3.iter().filter(|&&x| x == b'\n').count() + 
            packets4.iter().filter(|&&x| x == b'\n').count()
        );

        let quess = vec![
            Complete(Message::Update),
            Complete(Message::Update),
            Complete(Message::Remove),
            Complete(Message::Unknown),
            Complete(Message::Unknown),
            Complete(Message::Unknown),
            Complete(Message::Unknown),
            Complete(Message::Unknown),
            Complete(Message::Update),
            Complete(Message::Unknown),
            Complete(Message::Remove),
        ];
        for it in parser.iter().zip(quess.iter()) {
            assert_eq!(it.0, it.1);
        }
    }
}
