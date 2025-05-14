import { cookies } from 'next/headers';

export function GET(): Response {
  const userIds = cookies().get('user-ids')?.value;
  if (!userIds) return new Response(null, { status: 204 });
  try {
    return new Response(JSON.stringify(JSON.parse(userIds)), { status: 200 });
  } catch {
    return new Response(null, { status: 204 });
  }
}
