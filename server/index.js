import http from 'http';
import express from 'express';
import playfair from './playfair.js';

const app = express();

app.disable('x-powered-by');

app.use(express.urlencoded({ extended: true }));
app.use(express.json());

app.get('/', (req, res) => {
    console.log('GET /');

    const result = playfair(req.headers);

    if (!result) {
        console.log('   -> 400');
        res.sendStatus(400);
        return;
    }

    console.log('   -> 200');

    res.set('Content-Type', 'text/plain');
    res.send(result);
});

const server = http.createServer(app);
console.log('HTTP server initialized');

server.listen(5100, 'localhost');

console.log(`Magic is happening on http://localhost:5100\n`);