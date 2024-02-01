use std::io::{self, ErrorKind, Read, Write}; // Importando a livrario io em si (self)
use std::net::TcpStream; // Mesma coisa do server
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;

// Mesma coisa do server
const LOCAL: &str = "10.0.0.148:7777";
const MSG_SIZE: usize = 32;

fn main() {
  // Criando uma variável "cliente" mutável que é uma stream TCP e conectar ao IPv4 "LOCAL" (10.0.0.148:7777)
  let mut client = TcpStream::connect(LOCAL).expect("\x1b[31mStream failed to connect\x1b[0m");
  client.set_nonblocking(true).expect("\x1b[31mFailed to initiate non-blocking\x1b[0m"); // Mesma coisa do server

  let (tx, rx) = mpsc::channel::<String>(); // Mesma coisa do server

  // Isso aqui tudo é a mesma coisa do server, lê lá!
  thread::spawn(move || loop{
    let mut buff = vec![0; MSG_SIZE];
    match client.read_exact(&mut buff){
      Ok(_) => {
        // Faz uma iteração e checa se as mensagens dentro do buffer são iguais a zero
        // se forem a gente colecta dentro de um vetor e discarta elas (undercore = variável inexistente)
        let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
        println!("\x1b[32m> Message received in bytes\x1b[0m {:?}\n", msg);
      },
      Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
      Err(_) => {
        println!("\x1b[31m< Connection with server was severed\x1b[0m");
        break;
      }
    }
    // Ver se o server envia uma mensagem dizendo que ele recebeu a mensagem
    // que estamos enviando pelo client.
    // Se tivermos a resposta com a mensagem como um "Ok" iremos clonar a mensagem
    // em bytes e jogar numa variável buffer chamada "buff"
    match rx.try_recv() {
      Ok(msg) => {
        let mut buff = msg.clone().into_bytes(); // Variável buffer
        buff.resize(MSG_SIZE, 0); // Mudando o tamanho do buffer de acordo com o MSG_SIZE
        client.write_all(&buff).expect("\x1b[31mWriting to socket failed\x1b[0m"); // Escrevendo todos os buffers no client
        println!("\x1b[32m> Message sent\x1b[0m {:?}", msg); // Se não houver exceções printa a mensagem
      },
      // Checando se o TryRecieveError está vazio, e se tiver joga pra um unit type "()"
      // O Unit Type é usado quando não há outro valor significativo que possa ser retornado.
      // Ou seja, se tiver erro no recebimento, nada acontece
      Err(TryRecvError::Empty) => (),
      // Se é um tipo disconectado quebra pra fora do loop
      Err(TryRecvError::Disconnected) => break
    }
    // A thread irá dar sleep de 100 milisegundos
    thread::sleep(Duration::from_millis(100));
  });
  // Esse print será criado quando o usuário abrir o client
  println!("\x1b[32mWrite a message in the buffer:\x1b[0m");
    loop {
      // Criando uma string mutável
      let mut buff = String::new();

      // Lendo dentro daquela string usando o stdin
      // Basicamente, quando um usuário digita algo do console nós
      // iremos escrever isso na string "buff" para que seja do tipo string
      // Isso tudo dentro de um loop para que possa ser enviadas multiplas mensagens
      io::stdin().read_line(&mut buff).expect("\x1b[31mReading from stdin failed\x1b[0m");

      // Dando trim no buffer e jogando para dentro da variável msg que é do tipo string
      let msg = buff.trim().to_string();

      // Se a mensagem for igual à ":quit"
      // OU (||) se o "tx"."enviar mensagem" (tx = transmissor) vier com um error então irá quebrar do loop
      if msg == ":quit" || tx.send(msg).is_err() {break}
    }
    // Se alguém quebrar o loop essa mensagem será printada:
    println!("\x1b[33mChat disconnected... bye!\x1b[0m");
}
