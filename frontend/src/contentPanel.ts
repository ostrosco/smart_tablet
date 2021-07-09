export abstract class ContentPanel {
  public abstract get isSetUp(): boolean;

  public abstract setUp(): void;
  public abstract tearDown(): void;
  public abstract animationFrameTick(): void;
}