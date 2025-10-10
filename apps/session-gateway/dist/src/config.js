import { z } from 'zod';
const envSchema = z.object({
  PORT: z.coerce.number().int().min(0).default(3000),
  SCHEDULER_URL: z
    .string()
    .default('http://localhost:4000')
    .transform((value) => value.replace(/\/$/, '')),
  LOG_LEVEL: z
    .enum(['silent', 'fatal', 'error', 'warn', 'info', 'debug', 'trace'])
    .default('info'),
});
export const loadConfig = () => envSchema.parse(process.env);
