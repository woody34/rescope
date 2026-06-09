import { NestFactory } from '@nestjs/core';
import { AppModule } from './app.module';

async function bootstrap() {
  const app = await NestFactory.create(AppModule);

  // Allow the Angular dev server to call the API
  app.enableCors({
    origin: ['http://localhost:4444'],
    methods: ['GET', 'POST', 'DELETE', 'OPTIONS'],
    allowedHeaders: ['Content-Type', 'Authorization'],
  });

  const port = process.env.PORT ?? 3333;
  await app.listen(port);
  console.log(`[NestJS] Listening on http://localhost:${port}`);
}

void bootstrap();
