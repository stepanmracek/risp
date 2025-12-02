(begin
    (define pos 50)
    (define inputs (split-string (read-file "aoc2025/day1/input")))
    (define inputs (map (lambda (s) (list (substring s 0 1) (parse-int (substring s 1 -1)))) inputs))

    (define (dial direction amount)
        (begin
            (define zeros (/ amount 100))
            (define amount (mod amount 100))

            (define new-pos
                (if (= direction "R")
                    (+ pos amount)
                    (- pos amount)
                )
            )

            ; set zero to 1 if old position was not 0 and new position is outside range <1..99>
            (define zero
                (if
                    (and
                        (not (= pos 0))
                        (or (<= new-pos 0) (>= new-pos 100))
                    )
                1 0)
            )
            (set! zeros (+ zeros zero))

            (set! pos (mod new-pos 100))
            zeros
        )
    )

    (define zero-list (map (lambda (input) (apply dial input)) inputs))
    (define zero-count (apply + zero-list))
    (display zero-count)
)
