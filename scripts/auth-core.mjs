/** Start only the SuperTokens Core container for native services. */
import { runCompose } from './compose.mjs';
import { cancellation } from './test-process.mjs';

try {
  await runCompose({
    action: 'up',
    args: ['--detach', '--wait', 'supertokens'],
    files: ['docker-compose.yml', 'docker-compose.auth-local.yml'],
    keys: ['SUPERTOKENS_DATABASE_URL', 'SUPERTOKENS_API_KEY'],
    overrides: { COMPOSE_PROJECT_NAME: 'grocery-auth-core' },
    signal: cancellation(),
  });
} catch (error) {
  console.error(error.message);
  process.exitCode = 1;
}
