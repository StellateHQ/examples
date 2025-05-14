import { graphql } from 'gql.tada';
import { CartFragment } from './fragments';

export const AddToCartMutation = graphql(
  `
    mutation AddToCartMutation($productId: ID!, $quantity: Int!) {
      addToCart(productId: $productId, quantity: $quantity) {
        ...CartFragment
      }
    }
  `,
  [CartFragment]
);

export const RemoveFromCartMutation = graphql(
  `
    mutation RemoveFromCartMutation($productId: ID!, $quantity: Int) {
      removeFromCart(productId: $productId, quantity: $quantity) {
        ...CartFragment
      }
    }
  `,
  [CartFragment]
);
