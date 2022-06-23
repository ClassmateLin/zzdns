

#[derive(Debug)]
pub struct DNSHeader {
    tranction_id: u16,
    flags: u16,
    questions: u16,
    answer_rrs: u16,
    authority_rrs: u16,
    additional_rrs: u16,
}


impl DNSHeader {

    #[warn(dead_code)]
    pub fn parse(buf: &Vec<u8>) -> (DNSHeader, usize) {
        (
            DNSHeader{
                tranction_id: ((buf[0] as u16) << 8) | (buf[1] as u16),
                flags: ((buf[2] as u16) << 8) | (buf[3] as u16),
                questions: ((buf[4] as u16) << 8) | (buf[5] as u16),
                answer_rrs: ((buf[6] as u16) << 8) | (buf[7] as u16),
                authority_rrs: ((buf[8] as u16) << 8) | (buf[9] as u16),
                additional_rrs: ((buf[10] as u16) << 8) | (buf[11] as u16), 
            },
            12
        )
    }

    pub fn tranction_id(&self) -> u16 {
        self.tranction_id
    }

    pub fn questions(&self) -> u16{
        self.questions
    }

    pub fn answer_rrs(&self) -> u16 {
        self.answer_rrs
    }

    pub fn authority_rrs(&self) -> u16 {
        self.authority_rrs
    }

    pub fn additional_rrs(&self) -> u16 {
        self.additional_rrs
    }
    
    pub fn qr(&self) -> u16 {
        self.flags >> 15
    }

    pub fn op_code(&self) -> u16 {
        (self.flags >> 11)  % (1 << 4)
    }

    
    pub fn aa(&self) -> u16 {
        (self.flags >> 10) % (1 << 1)
    }

    pub fn tc(&self) -> u16 {
        (self.flags >> 9) % (1 << 1)
    }

    pub fn rd(&self) -> u16 {
        (self.flags >> 8) % (1 << 1)
    }

    pub fn ra(&self) -> u16 {
        (self.flags >> 7) % (1 << 1)
    }

    pub fn z(&self) -> u16 {
        (self.flags >> 4) % (1 << 3)
    }

    pub fn r_code(&self) -> u16 {
        self.flags % (1 << 4)
    } 
}


pub struct DNSQuestion {
    q_name: String,
    q_type: u16,
    q_class: u16,
}


impl DNSQuestion {

    pub fn parse(buf:Vec<u8>) -> (DNSQuestion, usize) {
        let mut data = Vec::<String>::new();
        let mut len = buf[0] as usize;
        let mut start = 1 as usize;
        let mut end = start + len;
        
        while len > 0 {
            data.push(String::from_utf8(buf[start..end].to_vec()).unwrap());
            start = end + 1;
            len = buf[end] as usize;
            end = start + len;
        }
        
        (
            DNSQuestion {
                q_name: data.join("."),
                q_type: ((buf[end + 1] as u16) << 8) | (buf[end + 2] as u16), 
                q_class: ((buf[end + 3] as u16) << 8) | (buf[end + 4] as u16), 
            },
            end + 4, 
        )
    }

    pub fn q_name(&self) -> &str {
        self.q_name.as_str()
    }

    pub fn q_type(&self) -> u16 {
        self.q_type
    }

    pub fn q_class(&self) -> u16 {
        self.q_class
    }
}


pub struct DNSResourceRecord{
    rr_name: String,
    rr_type: u16,
    rr_class: u16,
    rr_ttl: u32,
    rr_data_len: u16,
    rr_data: String,
}

impl DNSResourceRecord {
    pub fn parse(buf: Vec<u8>){
        println!("{:?}", buf);
    }
}