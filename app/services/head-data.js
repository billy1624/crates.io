import Service from '@ember/service';
import { tracked } from '@glimmer/tracking';

const DEFAULT_DESCRIPTION = 'rustacean.info';
const DEFAULT_CRATE_DESCRIPTION = 'A package for Rust.';

export default class HeadDataService extends Service {
  @tracked crate;

  get description() {
    return this.crate ? this.crate.description || DEFAULT_CRATE_DESCRIPTION : DEFAULT_DESCRIPTION;
  }
}
