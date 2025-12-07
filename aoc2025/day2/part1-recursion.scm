(begin
    (define pos 50)
    (define inputs (split-string-with (read-file "aoc2025/day2/input") ","))
    (define (parse-input s)
        (define strings (split-string-with s "-"))
        (map string->int strings)
    )
    (define inputs (map parse-input inputs))

    (define invalid-ids 0)
    (define (count-invalid-ids start stop)
        (define generator (make-generator start 1))

        (define (check-value generator)
            (define val (generator))
            (if (<= val stop)
                (begin
                    (define s (->string val))
                    (define l (length s))
                    (define left (substring s 0 (/ l 2)))
                    (define right (substring s (/ l 2) l))
                    (if (= left right)
                        (set! invalid-ids (+ invalid-ids val))
                        #f
                    )
                    (check-value generator)
                )
                #f
            )
        )

        (check-value generator)
    )

    (map (lambda (input) (apply count-invalid-ids input)) inputs)
    (display invalid-ids)
)
