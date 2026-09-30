declare module "*.wgsl?glsl" {
  export const vertex: string;
  export const fragment: string;
  export const uniformBlock: string;
  export const uniformStruct: string;
  export const samplers: string[];
}
