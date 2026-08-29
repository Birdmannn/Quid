import { Module } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import { ScheduleModule } from '@nestjs/schedule';
import { AppController } from './app.controller';
import { AppService } from './app.service';
import { AuthModule } from './auth/auth.module';
import { MissionsModule } from './missions/missions.module';
import { PrismaModule } from './prisma/prisma.module';
import { UploadModule } from './upload/upload.module';
import { IndexerModule } from './indexer/indexer.module';
import { UsersModule } from './users/users.module';

@Module({
  imports: [
    ConfigModule.forRoot({
      isGlobal: true,
      cache: true,
    }),
    ScheduleModule.forRoot(),
    PrismaModule,
    AuthModule,
    // Issue #331: exposes GET /users/me and PATCH /users/me/role so the
    // onboarding role choice is stored against the Prisma user.
    UsersModule,
    MissionsModule,
    UploadModule,
    IndexerModule,
  ],
  controllers: [AppController],
  providers: [AppService],
})
export class AppModule {}
