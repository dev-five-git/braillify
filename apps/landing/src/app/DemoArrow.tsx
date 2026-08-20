import { Flex, Image } from '@devup-ui/react'

/** 입력 박스와 출력 박스 사이의 방향 화살표. 한글·수학 데모가 공유한다. */
export function DemoArrow() {
  return (
    <Flex alignSelf="center">
      {/* 데스크톱에서만 보이는 원형 장식. 화살표와 중복 안내되지 않도록 장식으로 둔다. */}
      <Image
        alt=""
        display={['none', null, null, 'block']}
        mr="10px"
        role="presentation"
        src="/images/home/translate-arrow-circle.svg"
        w="16px"
      />
      <Image
        alt="점역 결과 방향"
        src="/images/home/translate-arrow.svg"
        transform={[null, null, null, 'rotate(-90deg)']}
        w={['16px', null, null, '24px']}
      />
    </Flex>
  )
}
