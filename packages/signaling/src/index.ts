import { SignalingServer } from './server.js';

const port = parseInt(process.env.SIGNALING_PORT ?? '3002', 10);

const server = new SignalingServer(port);
server.start();
