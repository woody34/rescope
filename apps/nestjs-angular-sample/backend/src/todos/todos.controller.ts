import {
  Controller,
  Get,
  Post,
  Delete,
  Param,
  Body,
  Req,
  UseGuards,
  HttpCode,
  NotFoundException,
} from '@nestjs/common';
import { Request } from 'express';
import { AuthGuard } from '../auth/auth.guard';
import { RolesGuard } from '../auth/roles.guard';
import { Roles } from '../auth/decorators';
import { DescopeToken } from '../auth/descope.service';
import { TodosService } from './todos.service';

type AuthRequest = Request & { user: DescopeToken };

@Controller('api/todos')
@UseGuards(AuthGuard, RolesGuard)
export class TodosController {
  constructor(private readonly todos: TodosService) {}

  @Get()
  findAll(@Req() req: AuthRequest) {
    return this.todos.findAll(req.user.sub, req.user.dct);
  }

  @Post()
  @HttpCode(201)
  create(@Body('text') text: string, @Req() req: AuthRequest) {
    return this.todos.create(text, req.user.sub, req.user.dct);
  }

  @Delete(':id')
  @Roles('admin')
  @HttpCode(204)
  remove(@Param('id') id: string, @Req() req: AuthRequest) {
    const deleted = this.todos.delete(id, req.user.sub, req.user.dct);
    if (!deleted) throw new NotFoundException(`Todo ${id} not found`);
  }
}
