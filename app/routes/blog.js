import Route from '@ember/routing/route';

export default class BlogRoute extends Route {
  setupController(controller) {
    if (!controller.hasData) {
      controller.fetchData();
    }
  }
}
