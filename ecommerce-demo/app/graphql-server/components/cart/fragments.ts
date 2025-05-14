import { graphql } from 'gql.tada';

export const CartItemFragment = graphql(`
  fragment CartItemFragment on CartItem {
    quantity
    product {
      id
      slug
      name
      price
      coverImage {
        src
        alt
      }
    }
  }
`);

export const CartModalFragment = graphql(
  `
    fragment CartModalFragment on Cart {
      id
      totalPrice
      items {
        quantity
        product {
          slug
        }
        ...CartItemFragment
      }
    }
  `,
  [CartItemFragment]
);

export const AddToCartFragment = graphql(`
  fragment AddToCartFragment on Product {
    id
    hasStock
  }
`);
