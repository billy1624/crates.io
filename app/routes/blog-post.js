import { NotFoundError } from '@ember-data/adapter/error';
import Route from '@ember/routing/route';
import { inject as service } from '@ember/service';
import { ajax_fail, ajax_get } from '../utils/ajax';
import { marked } from 'marked';

export default class BlogPostRoute extends Route {
  @service router;
  @service store;

  async model(params, transition) {
    let slug = params.slug;

    // console.log('_slug_', slug);

    const data = await ajax_fail(`https://raw.githubusercontent.com/SeaQL/rustacean.info/refs/heads/main/issues.json`);
    let post = data[slug] ?? {};

    let content = '';

    if (post.markdown) {
      content = await ajax_get(post.markdown);
      content = marked.parse(content);
    }

    // console.log('_post_', post);
    // console.log('_content_', content);

    post.content = content;

    return post;
  }

  setupController(controller, model) {
    super.setupController(...arguments);
  }

  resetController() {
    super.resetController(...arguments);
  }
}
