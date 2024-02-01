use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener; // Criar o servidor e escutar uma porta
use std::sync::mpsc; // Spawnar uma channel entre as conexões
use std::thread; // Trabalhar com várias threads

const LOCAL: &str = "10.0.0.148:7777"; // Sobre o "&str": doc.rust-lang.org/std/primitive.str.html
const MSG_SIZE: usize = 32; // Tamanho da mensagem em bytes, como 32 bits de exemplo

// Deixando os threads dormindo por 100 milisegundos entre cada um dos loops
fn sleep(){
  thread::sleep(::std::time::Duration::from_millis(100));
}

// Instanciando o server
fn main(){
  // Fazendo o vínculo da conexão TCP com o IPv4 referênciado na variável constante "LOCAL"
  // Se o vínculo (bind) falhar joga a mensagem de pânico abaixo VVVVVV
  let server = TcpListener::bind(LOCAL).expect("\x1b[31mListener failed to bind\x1b[0m");

  // Modo de "non-blocking" permite que nosso servidor cheque constantemente por mensagens
  server.set_nonblocking(true).expect("\x1b[31mFailed to initialize non-blocking\x1b[0m");

  // Criando um vetor mutável chamado "clients" que nos permitirá a conexão de múltiplos clientes
  // de uma vez só ao invés de apenas 1 ou 2
  let mut clients = vec![];

  // Instanciando nosso channel e assinando ele à um tipo string
  // Basicamente dizendo ao canal que várias strings serão enviadas para ele
  let (tx, rx) = mpsc::channel::<String>();
  // rx = the receiving half
  // tx = the sending half

  loop {
    // If let binding para extrair nossos resultados de saída do "server.accept"
    // O socket e o addr serão extraídos do server.accept
    // "server.accept" é o que nos permite aceitar conexões para esse servidor,
    // se tivermos um Ok então sinal que deu certo. O formato Ok() no rust é tipo um if.
    if let Ok((mut socket, addr)) = server.accept(){
      // O "socket" será a stream TCP que está se conectando e o "addr" será o endereço do socket.
      println!("\x1b[32m> Client\x1b[0m\x1b[33m {} \x1b[0m\x1b[32mconnected\x1b[0m", addr);

      // Clonando o TX (transmissor)
      let tx = tx.clone();

      // Tentando clonar o socket e botando ele no vetor "clients"
      // Se houver um erro a mensagem abaixo de pânico será exibida.
      clients.push(socket.try_clone().expect("\x1b[31mFailed to clone client\x1b[0m"));
      // A razão pela clonagem do socket é para que possamos utilizar com o thread

      // Spawnando a thread com o move closure e criando um loop onde um buffer mutável será
      // criado que será um vetor (vec!) com zeros e o tamanho da mensagem (MSG_SIZE).
      // Os encerramentos do Rust são funções anônimas que você pode salvar
      // em uma variável ou passar como argumentos para outras funções.
      thread::spawn(move || loop{
        let mut buff = vec![0; MSG_SIZE];

        // Isso irá ler a nossa mensagem no nosso buffer
        match socket.read_exact(&mut buff){
          Ok(_) => {
            // Atribuindo a variável "msg" ao nosso buffer.into_iter, ou seja pegar as mensagens que
            // foram recebidas e converte-las para um iterador e depois pegar os caracteres que não
            // são espaços em branco e coleta-los dentro do nosso vetor VVVVVVV
            let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();

            // Convertendo aqueles pedaços de strings em uma string final e atribuindo isso à variável "msg"
            let msg = String::from_utf8(msg).expect("\x1b[31mInvalid UTF8 message\x1b[0m");

            // Printando o endereço IPv4 que enviou a mensagem junto com a mensagem a seguir.
            // "msg" com uma flag de debug (:?)
            // doc.rust-lang.org/std/fmt/trait.Debug.html#examples
            println!("\x1b[32m{} -->\x1b[0m {:?}", addr, msg);

            // Enviando a mensagem através do transmissor (tx) até o receptor (rx)
            tx.send(msg).expect("\x1b[31mFailed to send message to reciever (rx)\x1b[0m");
          },
          // Checando o erro dentro do erro e se o tipo do erro (error kind)
          // for igual à um tipo de erro que poderia bloquear o nosso modo "non-blocking"
          // envia um unit type "=> ()", o parentesis é um unit type, ou seja, serve como um
          // "continue" ou como um "continue o programa normalmente"
          Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
          // Caso contrário iremos checar por outro erro e se tivermos um erro (não
          // importa o que tiver dentro dele, por isso o underscore "_") iremos apenas
          // fechar a conexão com o client
          Err(_) => {
            println!("\x1b[33m< Closing connection with:\x1b[0m {}\n", addr);
            break; // Quebrando pra fora do loop
          }
        }
        sleep(); // Chamando a função "sleep"
      });
    }
    // Quando nosso servidor receber uma mensagem isso irá acontecer:
    // Também tentaremos receber a msg através do canal

    // Coletando todas as mensagens que recebemos pelo canal e a variável "client"
    // que setamos como um vetor mutável será igual à um iterador dos clientes,
    // depois iremos filtrar os clientes e converter as mensagens em bytes #1 (continuação depois do #1)
    if let Ok(msg) = rx.try_recv(){
      clients = clients.into_iter().filter_map(|mut client| { // igual ao iterador dos clientes aqui
        let mut buff = msg.clone().into_bytes(); // conversão de bytes aqui #1
        buff.resize(MSG_SIZE, 0); // Mudando o tamanho do buffer de acordo com o tamanho da mensagem

        client.write_all(&buff).map(|_| client).ok() // Pegando o "client" e escrevendo todo o buffer e mapear até o cliente e enviar de volta
      }).collect::<Vec<_>>(); // Coletar tudo isso e jogar dentro de um vetor
    }
    sleep(); // Chamando a função sleep novamente
  }
}
