const http = require('http');

const server = http.createServer((_, res) => {
  res.writeHead(200, { 'Content-Type': 'text/plain' });
  res.end('Hello World!\n');
});

server.listen(3001, '127.0.0.1', () => {
  console.log('Listening on 127.0.0.1:3001');
});
