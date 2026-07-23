import { Box, Button, Flex } from '@devup-ui/react'
import Link from 'next/link'
import { HTMLAttributeAnchorTarget } from 'react'

export default function PillButton({
  href,
  target = '_self',
  children,
}: {
  href: string
  target?: HTMLAttributeAnchorTarget
  children: React.ReactNode
}) {
  return (
    <Link
      aria-label={`${children} link`}
      href={href}
      scroll={false}
      target={target}
    >
      <Button bg="transparent" border="none" cursor="pointer" data-group>
        <Flex
          alignItems="center"
          bg="#2B2B2B"
          borderRadius="1000px"
          justifyContent="space-between"
          px={['20px', null, null, '40px']}
          py={['16px', null, null, '20px']}
          selectors={{
            '& span': {
              alignSelf: 'center',
            },
          }}
          w={['240px', null, null, '300px']}
        >
          {children}
          <Box boxSize="16px" position="relative">
            <Box
              _groupActive={{
                bg: 'url(/images/home/button-arrow.svg) #FFF',
                bgPosition: 'center',
                boxSize: '32px',
                bgSize: 'contain',
              }}
              _groupHover={{
                boxSize: '40px',
                bg: 'url(/images/home/button-arrow.svg) #FFF',
                bgPosition: 'center',
                bgSize: 'contain',
              }}
              aspectRatio="1"
              bg="url(/images/home/button-arrow.svg) #FFF"
              bgRepeat="no-repeat"
              bgSize="0%"
              borderRadius="50%"
              boxSize="100%"
              left="50%"
              position="absolute"
              top="50%"
              transform="translate(-50%, -50%)"
              transition="all 0.2s ease"
            />
          </Box>
        </Flex>
      </Button>
    </Link>
  )
}
