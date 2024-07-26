import { module, test } from 'qunit';

import fetch from 'fetch';

import { setupTest } from '../../../helpers';
import setupMirage from '../../../helpers/setup-mirage';

module('Mirage | GET /api/v1/crates/:id/versions', function (hooks) {
  setupTest(hooks);
  setupMirage(hooks);

  test('returns 404 for unknown crates', async function (assert) {
    let response = await fetch('https://crates.io/api/v1/crates/foo/versions');
    assert.strictEqual(response.status, 404);
    assert.deepEqual(await response.json(), { errors: [{ detail: 'Not Found' }] });
  });

  test('empty case', async function (assert) {
    this.server.create('crate', { name: 'rand' });

    let response = await fetch('https://crates.io/api/v1/crates/rand/versions');
    assert.strictEqual(response.status, 200);
    assert.deepEqual(await response.json(), {
      versions: [],
    });
  });

  test('returns all versions belonging to the specified crate', async function (assert) {
    let user = this.server.create('user');
    let crate = this.server.create('crate', { name: 'rand' });
    this.server.create('version', { crate, num: '1.0.0' });
    this.server.create('version', { crate, num: '1.1.0', publishedBy: user });
    this.server.create('version', { crate, num: '1.2.0', rust_version: '1.69' });

    let response = await fetch('https://crates.io/api/v1/crates/rand/versions');
    assert.strictEqual(response.status, 200);
    assert.deepEqual(await response.json(), {
      versions: [
        {
          id: '1',
          crate: 'rand',
          crate_size: 0,
          created_at: '2010-06-16T21:30:45Z',
          dl_path: 'https://crates.io/api/v1/crates/rand/1.0.0/download',
          downloads: 0,
          license: 'MIT/Apache-2.0',
          links: {
            dependencies: 'https://crates.io/api/v1/crates/rand/1.0.0/dependencies',
            version_downloads: 'https://crates.io/api/v1/crates/rand/1.0.0/downloads',
          },
          num: '1.0.0',
          published_by: null,
          readme_path: 'https://crates.io/api/v1/crates/rand/1.0.0/readme',
          rust_version: null,
          updated_at: '2017-02-24T12:34:56Z',
          yanked: false,
        },
        {
          id: '2',
          crate: 'rand',
          crate_size: 162_963,
          created_at: '2010-06-16T21:30:45Z',
          dl_path: 'https://crates.io/api/v1/crates/rand/1.1.0/download',
          downloads: 3702,
          license: 'MIT',
          links: {
            dependencies: 'https://crates.io/api/v1/crates/rand/1.1.0/dependencies',
            version_downloads: 'https://crates.io/api/v1/crates/rand/1.1.0/downloads',
          },
          num: '1.1.0',
          published_by: {
            id: 1,
            avatar: 'https://avatars1.githubusercontent.com/u/14631425?v=4',
            login: 'user-1',
            name: 'User 1',
            url: 'https://github.com/user-1',
          },
          readme_path: 'https://crates.io/api/v1/crates/rand/1.1.0/readme',
          rust_version: null,
          updated_at: '2017-02-24T12:34:56Z',
          yanked: false,
        },
        {
          id: '3',
          crate: 'rand',
          crate_size: 325_926,
          created_at: '2010-06-16T21:30:45Z',
          dl_path: 'https://crates.io/api/v1/crates/rand/1.2.0/download',
          downloads: 7404,
          license: 'Apache-2.0',
          links: {
            dependencies: 'https://crates.io/api/v1/crates/rand/1.2.0/dependencies',
            version_downloads: 'https://crates.io/api/v1/crates/rand/1.2.0/downloads',
          },
          num: '1.2.0',
          published_by: null,
          readme_path: 'https://crates.io/api/v1/crates/rand/1.2.0/readme',
          rust_version: '1.69',
          updated_at: '2017-02-24T12:34:56Z',
          yanked: false,
        },
      ],
    });
  });
});
