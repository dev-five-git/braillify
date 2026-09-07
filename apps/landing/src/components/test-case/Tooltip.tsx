'use client'

import { keyframes, VStack } from '@devup-ui/react'
import { ComponentProps, useRef, useState } from 'react'
import { useEffect } from 'react'

const fadeIn = keyframes({
  from: {
    opacity: 0,
  },
  to: {
    opacity: 1,
  },
})

export default function Tooltip({
  translateX = '0px',
  translateY = '10px',
  ...props
}: Omit<ComponentProps<typeof VStack<'div'>>, 'as'> & {
  translateX?: string
  translateY?: string
}) {
  const [viewportWidth, setViewportWidth] = useState(0)
  const ref = useRef<HTMLDivElement>(null)
  if (typeof window !== 'undefined' && viewportWidth !== window.innerWidth)
    setViewportWidth(window.innerWidth)
  useEffect(() => {
    const handleResize = () => {
      setViewportWidth(window.innerWidth)
    }

    window.addEventListener('resize', handleResize)

    return () => {
      window.removeEventListener('resize', handleResize)
    }
  }, [])

  return (
    <VStack
      ref={(el) => {
        if (!el) return

        ref.current = el

        const mo = new ResizeObserver((entries) => {
          entries.forEach((entry) => {
            const target = entry.target as HTMLDivElement
            const { x, width } = target.getBoundingClientRect()
            if (x + width > viewportWidth) {
              target.style.setProperty(
                '--translateX',
                `-${x + width - viewportWidth + 16}px`,
              )
            }
          })
        })

        mo.observe(el)

        return () => {
          mo.disconnect()
          el.style.setProperty('--translateX', translateX)
        }
      }}
      _groupHover={{
        display: 'flex',
        animationName: fadeIn,
        animationDuration: '0.2s',
        animationFillMode: 'forwards',
      }}
      aria-label="tooltip"
      bg="rgba(0, 0, 0, 0.75)"
      borderRadius="4px"
      display="none"
      justifyContent="center"
      maxW="calc(100vw - 32px)"
      onMouseEnter={(e) => {
        e.stopPropagation()
      }}
      onMouseLeave={(e) => {
        e.stopPropagation()
      }}
      opacity={0}
      pos="absolute"
      px="10px"
      py="8px"
      styleOrder={1}
      styleVars={{
        translateY: translateY,
        translateX: translateX,
      }}
      transform="translate(var(--translateX, 0px), var(--translateY, 10px))"
      zIndex="10"
      {...props}
    />
  )
}
