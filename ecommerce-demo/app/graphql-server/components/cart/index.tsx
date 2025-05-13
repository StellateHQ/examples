import { graphql } from 'gql.tada';
import { gql } from 'lib/gql';
import { CartModalFragment } from './fragments';
import CartModal from './modal';

const CartQuery = graphql(
  `
    query CartQuery {
      currentUser {
        id
        cart {
          ...CartModalFragment
        }
      }
    }
  `,
  [CartModalFragment]
);

export default async function Cart() {
  const { data } = await gql(CartQuery, {});

  return <CartModal data={data?.currentUser?.cart ?? null} />;
}
