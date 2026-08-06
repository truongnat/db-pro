export type Factory<T> = (container: DIContainer) => T;

interface Registration<T> {
  factory: Factory<T>;
  singleton: boolean;
}

export class DIContainer {
  private factories = new Map<string, Registration<unknown>>();
  private singletons = new Map<string, unknown>();
  private resolving = new Set<string>();
  private frozen = false;

  register<T>(
    name: string,
    factory: Factory<T>,
    singleton = true,
  ): void {
    if (this.frozen) {
      throw new Error(
        `Cannot register "${name}": container is frozen after bootstrap`,
      );
    }
    this.factories.set(name, { factory, singleton });
  }

  resolve<T>(name: string): T {
    if (this.singletons.has(name)) {
      return this.singletons.get(name) as T;
    }

    const registration = this.factories.get(name);
    if (!registration) {
      throw new Error(`Service "${name}" is not registered`);
    }

    if (this.resolving.has(name)) {
      throw new Error(
        `Circular dependency detected while resolving "${name}"`,
      );
    }

    this.resolving.add(name);
    try {
      const instance = registration.factory(this);

      if (registration.singleton) {
        this.singletons.set(name, instance);
      }

      return instance as T;
    } finally {
      this.resolving.delete(name);
    }
  }

  freeze(): void {
    this.frozen = true;
  }

  has(name: string): boolean {
    return this.factories.has(name) || this.singletons.has(name);
  }
}
