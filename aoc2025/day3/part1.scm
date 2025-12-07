(begin
    (define lines (split-string-with (read-file "aoc2025/day3/input") "\n"))

    (define (process-line line)
        (begin
            (define max 0)
            (define len (length line))
            (do
                ((i 0 (+ i 1)))
                ((>= i (- len 1)))

                (define v1 (* 10 (string->int (string-ref line i))))

                (do
                    ((j (+ i 1) (+ j 1)))
                    ((>= j len))

                    (define v2 (string->int (string-ref line j)))
                    (define candidate (+ v1 v2))
                    (if
                        (> candidate max)
                        (set! max candidate)
                        #f
                    )
                )
            )
            (display line " -> " max "\n")
            max
        )
    )

    (display (apply + (map process-line lines)))
)
