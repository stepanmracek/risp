(begin
  (define fact (lambda (n) (if (<= n 1) 1 (* n (fact (- n 1))))))
  (define ans (fact 20))
  (display ans))
