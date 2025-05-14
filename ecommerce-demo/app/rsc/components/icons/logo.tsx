import clsx from 'clsx';

export default function LogoIcon(props: React.ComponentProps<'svg'>) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 32 36"
      fill="none"
      {...props}
      className={clsx('h-4 w-4 fill-black dark:fill-white', props.className)}
    >
      <g clipPath="url(#a)" fillRule="evenodd" clipRule="evenodd">
        <path
          d="m30.208 28.318.025-.015a2.54 2.54 0 0 0 1.138-1.5c.174-.628.1-1.3-.205-1.875l-2.905-5.536L6.38 30.98l7.827 4.539a3.573 3.573 0 0 0 3.584 0l12.416-7.2ZM3.94 22.347 21.494 2.628 17.792.482a3.573 3.573 0 0 0-3.584 0l-12.416 7.2A3.612 3.612 0 0 0 0 10.8v2.7c0 1.67.35 3.321 1.03 4.845a11.862 11.862 0 0 0 2.91 4v.002Z"
          fill="#4568A5"
        />
        <path
          d="m30.208 7.682-.419-.243-10.016 9.25 8.4 7.313c1.325 1.152 1.816 2.833 1.304 4.743l.731-.424a3.592 3.592 0 0 0 1.313-1.32c.315-.548.48-1.17.48-1.802v-14.4a3.612 3.612 0 0 0-1.793-3.117ZM15.57 20.637 3.996 11.67c-1.304-1.016-1.893-2.536-1.625-4.322l-.579.335A3.612 3.612 0 0 0 0 10.8v14.4c0 .633.164 1.255.479 1.803a3.591 3.591 0 0 0 1.313 1.32l3.315 1.922 10.463-9.608Z"
          fill="#608BD8"
        />
      </g>
      <defs>
        <clipPath id="a">
          <path fill="#fff" d="M0 0h32v36H0z" />
        </clipPath>
      </defs>
    </svg>
  );
}
