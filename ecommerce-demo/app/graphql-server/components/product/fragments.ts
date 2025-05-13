import { graphql } from 'gql.tada';

export const GalleryFragment = graphql(`
  fragment GalleryFragment on Product {
    coverImage {
      src
      alt
    }
    images {
      src
      alt
    }
  }
`);
